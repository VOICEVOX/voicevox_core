use std::{cell::UnsafeCell, collections::BTreeMap, ptr::NonNull, sync::Mutex};

/// Cの世界に貸し出す`[u8]`の所有者(owner)。
///
/// `Mutex`による内部可変性を持ち、すべての操作は共有参照から行うことができる。
///
/// # Motivation
///
/// 本クレートが提供するAPIとして、バイト列の生成(create)とその解放(free)がある。APIとしては"生成
/// "時に`Box<[u8]>`のownershipがC側に渡され、"解放"時にはそのownershipがRust側に返されるといった形
/// となる。
///
/// しかし実装としては`Box<impl Sized>`の場合とは異なり、何かしらの情報をRust側で保持し続けなくては
/// ならない。実態としてはRust側がバッファの所有者(owner)であり続け、C側にはその参照が渡される形にな
/// る。この構造体はその"所有者"であり、実際にRustのオブジェクトを保持し続ける。
pub(crate) static U8_SLICE_OWNER: SliceOwner<u8> = SliceOwner::new();

pub(crate) struct SliceOwner<T> {
    slices: Mutex<BTreeMap<usize, UnsafeCell<Box<[T]>>>>,
}

impl<T> SliceOwner<T> {
    const fn new() -> Self {
        Self {
            slices: Mutex::new(BTreeMap::new()),
        }
    }

    /// `Box<[T]>`を所有し、その先頭ポインタと長さを参照としてC API利用者に与える。
    ///
    /// # Safety
    ///
    /// - `out_ptr`は書き込みについて[有効]でなければならない。
    /// - `out_len`は書き込みについて[有効]でなければならない。
    ///
    /// [有効]: https://doc.rust-lang.org/stable/std/ptr/index.html#safety
    pub(crate) unsafe fn own_and_lend(
        &self,
        slice: impl Into<Box<[T]>>,
        out_ptr: NonNull<*mut T>,
        out_len: NonNull<usize>,
    ) {
        let mut slices = self.slices.lock().unwrap();

        let slice = slice.into();
        let ptr = slice.as_ptr() as *mut T;
        let len = slice.len();

        let duplicated = slices.insert(ptr as usize, slice.into()).is_some();
        assert!(!duplicated, "duplicated");

        out_ptr.as_ptr().write_unaligned(ptr);
        out_len.as_ptr().write_unaligned(len);
    }

    /// `own_and_lend`でC API利用者に貸し出したポインタに対応する`Box<[u8]>`をデストラクトする。
    ///
    /// # Panics
    ///
    /// `ptr`が`own_and_lend`で貸し出されたポインタではないとき、パニックする。
    pub(crate) fn drop_for(&self, ptr: *mut T) {
        let mut slices = self.slices.lock().unwrap();

        slices.remove(&(ptr as usize)).expect(
            "解放しようとしたポインタはvoicevox_coreの管理下にありません。\
             誤ったポインタであるか、二重解放になっていることが考えられます",
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::MaybeUninit, ptr::NonNull};

    use super::SliceOwner;

    #[test]
    fn it_works() {
        lend_and_delete(vec::<()>(0, &[]));
        lend_and_delete(vec(0, &[()]));
        lend_and_delete(vec(2, &[()]));

        lend_and_delete(vec::<u8>(0, &[]));
        lend_and_delete(vec(0, &[0u8]));
        lend_and_delete(vec(2, &[0u8]));

        lend_and_delete(vec::<f32>(0, &[]));
        lend_and_delete(vec(0, &[0f32]));
        lend_and_delete(vec(2, &[0f32]));

        fn lend_and_delete<T>(vec: Vec<T>) {
            let owner = SliceOwner::<T>::new();
            let expected_len = vec.len();
            let (ptr, len) = unsafe {
                let mut ptr = MaybeUninit::uninit();
                let mut len = MaybeUninit::uninit();
                owner.own_and_lend(
                    vec,
                    NonNull::new(ptr.as_mut_ptr()).unwrap(),
                    NonNull::new(len.as_mut_ptr()).unwrap(),
                );
                (ptr.assume_init(), len.assume_init())
            };
            assert_eq!(expected_len, len);
            owner.drop_for(ptr);
        }

        fn vec<T: Clone>(initial_cap: usize, elems: &[T]) -> Vec<T> {
            let mut vec = Vec::with_capacity(initial_cap);
            vec.extend_from_slice(elems);
            vec
        }
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn it_denies_unknown_ptr() {
        let owner = SliceOwner::<i32>::new();
        let x = 42;
        owner.drop_for(&x as *const i32 as *mut i32);
    }
}
