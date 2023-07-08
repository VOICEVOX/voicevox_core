use std::{
    collections::BTreeSet,
    ffi::{c_char, CStr, CString},
    sync::Mutex,
};

/// dropして良い`*mut c_char`を把握し、チェックする。
///
/// `Mutex`による内部可変性を持ち、すべての操作は共有参照から行うことができる。
///
/// # Motivation
///
/// `CString`は`Box<impl Sized>`と同様Cの世界でもポインタ一つで実体を表すことができるため、こちら側
/// で管理すべきものは本来無い。しかしながら本クレートが提供するAPIには「解放不要」な文字列を返すも
/// のが含まれている。ユーザーが誤ってそのような文字列を解放するのは未定義動作 (undefined behavior)
/// であるため、綺麗にSEGVするとも限らない。`once_cell::sync::Lazy`由来の文字列の場合、最悪解放が成
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
    owned_str_addrs: BTreeSet<usize>,
    static_str_addrs: BTreeSet<usize>,
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

        let duplicated = !owned_str_addrs.insert(s.as_ptr() as usize);
        assert!(!duplicated, "duplicated");
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

        static_str_addrs.insert(s.as_ptr() as usize);
        s
    }

    /// `*mut c_char`が`whitelist`を通ったものかどうかチェックする。
    ///
    /// # Panics
    ///
    /// `ptr`が`Self::whitelist`を経由したものではないならパニックする。
    pub(crate) fn check(&self, ptr: *mut c_char) -> *mut c_char {
        let Inner {
            owned_str_addrs,
            static_str_addrs,
            ..
        } = &mut *self.0.lock().unwrap();

        let addr = ptr as usize;
        if !owned_str_addrs.remove(&addr) {
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
        ptr
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{c_char, CStr};

    use super::CStringDropChecker;

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn it_denies_unknown_char_ptr() {
        let checker = CStringDropChecker::new();
        let s = CStr::from_bytes_with_nul(b"\0").unwrap().to_owned();
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

        static STATIC: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };
    }
}
