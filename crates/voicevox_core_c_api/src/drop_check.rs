use std::{
    collections::BTreeSet,
    ffi::{CStr, CString, c_char},
    num::NonZero,
    ptr::NonNull,
    sync::Mutex,
};

use easy_ext::ext;

/// dropして良い`*mut c_char`を把握し、チェックする。
///
/// `Mutex`による内部可変性を持ち、すべての操作は共有参照から行うことができる。
///
/// # Motivation
///
/// `CString`は`Box<impl Sized>`と同様Cの世界でもポインタ一つで実体を表すことができるため、こちら側
/// で管理すべきものは本来無い。しかしながら本クレートが提供するAPIには「解放不要」な文字列を返すも
/// のが含まれている。ユーザーが誤ってそのような文字列を解放するのは未定義動作 (undefined behavior)
/// であるため、綺麗にSEGVするとも限らない。`std::sync::LazyLock`由来の文字列の場合、最悪解放が成
/// 功してしまう。
///
/// この構造体はCの世界から帰ってきた`*mut c_char`を`CString`としてdropする際、それが本当にこちら側
/// が送り出した`CString`かどうかをチェックする。
///
/// Cの世界に`CString`を送り出す前に`whitelist`を通し、戻って来た`*mut c_char`を`CString`にしてdrop
/// する前に`check`に通す。
pub(crate) static C_STRING_DROP_CHECKER: CStringDropChecker = CStringDropChecker::new();

pub(crate) struct CStringDropChecker(Mutex<Inner>);

struct Inner {
    owned_str_addrs: BTreeSet<NonZero<usize>>,
    static_str_addrs: BTreeSet<NonZero<usize>>,
}

impl CStringDropChecker {
    const fn new() -> Self {
        Self(Mutex::new(Inner {
            owned_str_addrs: BTreeSet::new(),
            static_str_addrs: BTreeSet::new(),
        }))
    }

    /// `CString`をホワイトリストに追加する。
    ///
    /// Cの世界に`CString`を送り出す前にこの関数を挟む。
    pub(crate) fn whitelist(&self, s: CString) -> CString {
        let Inner {
            owned_str_addrs, ..
        } = &mut *self.0.lock().unwrap();

        let ptr = s.as_non_null_ptr();
        let duplicated = !owned_str_addrs.insert(ptr.addr());
        if duplicated {
            panic!(
                "別の{ptr:p}が管理下にあります。原因としては以前に別の文字列が{ptr:p}として存在\
                 しており、それが誤った形で解放されたことが考えられます。このライブラリで生成した\
                 オブジェクトの解放は、このライブラリが提供するAPIで行われなくてはなりません",
            );
        }
        s
    }

    /// `&'static CStr`をブラックリストに追加する。
    ///
    /// Cの世界に`Lazy`由来の`&'static CStr`を送り出す前にこの関数を挟む。
    ///
    /// ホワイトリストとブラックリストは重複しないと考えてよく、ブラックリストはエラーメセージの制御
    /// のためのみに使われる。
    pub(crate) fn blacklist(&self, s: &'static CStr) -> &'static CStr {
        let Inner {
            static_str_addrs, ..
        } = &mut *self.0.lock().unwrap();

        static_str_addrs.insert(s.as_non_null_ptr().addr());
        s
    }

    /// `*mut c_char`が`whitelist`を通ったものかどうかチェックする。
    ///
    /// ヌルポインタを許容する。
    ///
    /// # Panics
    ///
    /// `ptr`が非ヌルで、`Self::whitelist`を経由したものではないならパニックする。
    pub(crate) fn check(&self, ptr: *mut c_char) -> Option<NonNull<c_char>> {
        let Inner {
            owned_str_addrs,
            static_str_addrs,
            ..
        } = &mut *self.0.lock().unwrap();

        if let Some(addr) = NonZero::new(ptr.addr())
            && !owned_str_addrs.remove(&addr)
        {
            if static_str_addrs.contains(&addr) {
                panic!(
                    "解放しようとしたポインタはvoicevox_core管理下のものですが、\
                     voicevox_coreがアンロードされるまで永続する文字列に対するものです。\
                     解放することはできません",
                )
            }
            panic!(
                "解放しようとしたポインタはvoicevox_coreの管理下にありません。\
                 誤ったポインタであるか、二重解放になっていることが考えられます",
            );
        }
        NonNull::new(ptr)
    }
}

#[ext]
impl CStr {
    fn as_non_null_ptr(&self) -> NonNull<c_char> {
        NonNull::new(self.as_ptr() as *mut c_char).expect("comes from a `CStr`")
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{CStr, c_char},
        ptr,
    };

    use super::CStringDropChecker;

    #[test]
    fn it_accepts_null() {
        let checker = CStringDropChecker::new();
        checker.check(ptr::null_mut());
    }

    #[test]
    #[should_panic(
        expected = "このライブラリで生成したオブジェクトの解放は、このライブラリが提供するAPIで\
                    行われなくてはなりません"
    )]
    fn it_denies_duplicated_char_ptr() {
        let checker = CStringDropChecker::new();
        let s = c"".to_owned();
        checker.whitelist(checker.whitelist(s));
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn it_denies_unknown_char_ptr() {
        let checker = CStringDropChecker::new();
        let s = c"".to_owned();
        checker.check(s.into_raw());
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_core管理下のものですが、voicevox_coreがアンロードされるまで永続する文字列に対するものです。解放することはできません"
    )]
    fn it_denies_known_static_char_ptr() {
        let checker = CStringDropChecker::new();
        checker.blacklist(STATIC);
        checker.check(STATIC.as_ptr() as *mut c_char);

        static STATIC: &CStr = c"";
    }
}
