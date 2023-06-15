use std::{
    cell::UnsafeCell,
    collections::{BTreeMap, BTreeSet},
    ffi::{c_char, CStr, CString},
    mem::MaybeUninit,
    sync::Mutex,
};

pub(crate) static C_STRING_OWNER: CStringOwner = CStringOwner::new();

pub(crate) struct CStringOwner(Mutex<CStringOwnerInner>);

struct CStringOwnerInner {
    owned_c_strings: BTreeMap<usize, UnsafeCell<CString>>,
    static_str_addrs: BTreeSet<usize>,
}

impl CStringOwner {
    const fn new() -> Self {
        Self(Mutex::new(CStringOwnerInner {
            owned_c_strings: BTreeMap::new(),
            static_str_addrs: BTreeSet::new(),
        }))
    }

    pub(crate) fn own_and_lend(&self, s: CString, out: &mut MaybeUninit<*mut c_char>) {
        let CStringOwnerInner {
            owned_c_strings, ..
        } = &mut *self.0.lock().unwrap();

        let ptr = s.as_ptr() as *mut c_char;

        let duplicated = owned_c_strings.insert(ptr as usize, s.into()).is_some();
        assert!(!duplicated, "duplicated");

        out.write(ptr);
    }

    /// `c_string_into_raw`でC API利用側に貸し出したポインタに対し、デアロケートする。
    ///
    /// # Safety
    ///
    /// - `ptr`は`c_string_into_raw`で取得したものであること。
    pub(crate) fn delete(&self, ptr: *mut c_char) {
        let CStringOwnerInner {
            owned_c_strings,
            static_str_addrs,
            ..
        } = &mut *self.0.lock().unwrap();

        let addr = ptr as usize;
        if owned_c_strings.remove(&addr).is_none() {
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
    }

    pub(crate) fn memorize_static(&self, s: &'static CStr) -> *const c_char {
        let CStringOwnerInner {
            static_str_addrs, ..
        } = &mut *self.0.lock().unwrap();

        let ptr = s.as_ptr();
        static_str_addrs.insert(ptr as usize);
        ptr
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{c_char, CStr};

    use super::CStringOwner;

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn it_denies_unknown_char_ptr() {
        let owner = CStringOwner::new();
        let s = CStr::from_bytes_with_nul(b"\0").unwrap().to_owned();
        owner.delete(s.into_raw());
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_core管理下のものですが、voicevox_coreがアンロードされるまで永続する文字列に対するものです。解放することはできません"
    )]
    fn it_denies_known_static_char_ptr() {
        let owner = CStringOwner::new();
        owner.memorize_static(STATIC);
        owner.delete(STATIC.as_ptr() as *mut c_char);

        static STATIC: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };
    }
}
