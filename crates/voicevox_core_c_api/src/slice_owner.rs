use std::{
    cell::UnsafeCell, collections::BTreeMap, mem::MaybeUninit, num::NonZeroUsize, ptr::NonNull,
    sync::Mutex,
};

use tracing::warn;

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
    non_empty_slices: Mutex<BTreeMap<NonZeroUsize, UnsafeCell<Box<[T]>>>>,
}

impl<T: SliceElement> SliceOwner<T> {
    const fn new() -> Self {
        Self {
            non_empty_slices: Mutex::new(BTreeMap::new()),
        }
    }

    /// 与えられた`Box<[T]>`が空ではないならそれを所有し、その先頭ポインタと長さを参照としてC API利用者に与える。
    ///
    /// 空のときは[`SliceElement::REF_FOR_EMPTY`]を返す。
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
        out_ptr: NonNull<NonNull<T>>,
        out_len: NonNull<usize>,
    ) {
        let slice = slice.into();

        if slice.is_empty() {
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe { out_ptr.write_unaligned(T::PTR_FOR_EMPTY) };
            unsafe { out_len.write_unaligned(0) };
            return;
        }

        let mut slices = self.non_empty_slices.lock().unwrap();

        let ptr = NonNull::new(slice.as_ptr() as *mut T).expect("comes from a slice");
        let len = slice.len();

        let duplicated = slices.insert(ptr.addr(), slice.into()).is_some();
        if duplicated {
            panic!(
                "別の{ptr:p}が管理下にあります。原因としては以前に別の配列が{ptr:p}として存在\
                 しており、それが誤った形で解放されたことが考えられます。このライブラリで生成した\
                 オブジェクトの解放は、このライブラリが提供するAPIで行われなくてはなりません",
            );
        }

        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { out_ptr.write_unaligned(ptr) };
        unsafe { out_len.write_unaligned(len) };
    }

    /// `own_and_lend`でC API利用者に貸し出したポインタに対応する`Box<[u8]>`をデストラクトする。
    ///
    /// ヌルポインタに対しては何もしない。[`SliceElement::REF_FOR_EMPTY`]に対しては警告のみ出して何もしない。
    ///
    /// # Panics
    ///
    /// `ptr`が非ヌルで、かつ`own_and_lend`で貸し出されたポインタではないとき、パニックする。
    pub(crate) fn drop_for(&self, ptr: *mut T) {
        let Some(ptr) = NonNull::new(ptr) else { return };

        if ptr == T::PTR_FOR_EMPTY {
            warn!("`{}`を解放することはできません", T::EMPTY_SLICE_VAR_NAME);
            return;
        }

        self.non_empty_slices
            .lock()
            .unwrap()
            .remove(&ptr.addr())
            .expect(
                "解放しようとしたポインタはvoicevox_coreの管理下にありません。\
                 誤ったポインタであるか、二重解放になっていることが考えられます",
            );
    }
}

pub(crate) trait SliceElement: Sized + 'static {
    const EMPTY_SLICE_VAR_NAME: &str;
    const REF_FOR_EMPTY: &'static MaybeUninit<Self>;

    const PTR_FOR_EMPTY: NonNull<Self> =
        NonNull::new(Self::REF_FOR_EMPTY.as_ptr() as *mut _).unwrap();
}

impl SliceElement for u8 {
    const EMPTY_SLICE_VAR_NAME: &str = "voicevox_empty_bytes";
    const REF_FOR_EMPTY: &'static MaybeUninit<Self> = {
        static DUMMY: MaybeUninit<u8> = MaybeUninit::uninit();
        &DUMMY
    };
}

#[cfg(test)]
mod tests {
    use std::{
        mem::MaybeUninit,
        ptr::{self, NonNull},
    };

    use super::{SliceElement, SliceOwner};

    #[test]
    fn it_works() {
        lend_and_delete(vec::<u8>(0, &[]));
        lend_and_delete(vec(0, &[0u8]));
        lend_and_delete(vec(2, &[0u8]));

        fn lend_and_delete<T: SliceElement>(vec: Vec<T>) {
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
            owner.drop_for(ptr.as_ptr());
        }

        fn vec<T: Clone>(initial_cap: usize, elems: &[T]) -> Vec<T> {
            let mut vec = Vec::with_capacity(initial_cap);
            vec.extend_from_slice(elems);
            vec
        }
    }

    #[test]
    fn it_accepts_null() {
        let owner = SliceOwner::<u8>::new();
        owner.drop_for(ptr::null_mut());
    }

    #[test]
    fn it_accepts_empty() {
        let owner = SliceOwner::<u8>::new();

        let (ptr, len) = unsafe {
            let mut ptr = MaybeUninit::uninit();
            let mut len = MaybeUninit::uninit();
            owner.own_and_lend(
                [],
                NonNull::new(ptr.as_mut_ptr()).unwrap(),
                NonNull::new(len.as_mut_ptr()).unwrap(),
            );
            (ptr.assume_init(), len.assume_init())
        };

        assert_eq!(0, len);

        for _ in 0..2 {
            owner.drop_for(ptr.as_ptr());
        }
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn it_denies_unknown_ptr() {
        let owner = SliceOwner::<u8>::new();
        let mut x = 42;
        owner.drop_for(&raw mut x);
    }
}
