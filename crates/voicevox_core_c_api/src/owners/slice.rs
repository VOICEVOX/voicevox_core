use std::{cell::UnsafeCell, collections::BTreeMap, mem::MaybeUninit, sync::Mutex};

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

    pub(crate) fn own_and_lend(
        &self,
        slice: impl Into<Box<[T]>>,
        out_ptr: &mut MaybeUninit<*mut T>,
        out_len: &mut MaybeUninit<usize>,
    ) {
        let mut slices = self.slices.lock().unwrap();

        let slice = slice.into();
        let ptr = slice.as_ptr() as *mut T;
        let len = slice.len();

        slices.insert(ptr as usize, slice.into());
        out_ptr.write(ptr);
        out_len.write(len);
    }

    pub(crate) fn delete(&self, ptr: *mut T) {
        let mut slices = self.slices.lock().unwrap();

        slices.remove(&(ptr as usize)).expect(
            "解放しようとしたポインタはvoicevox_coreの管理下にありません。\
             誤ったポインタであるか、二重解放になっていることが考えられます",
        );
    }
}

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;

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
                owner.own_and_lend(vec, &mut ptr, &mut len);
                (ptr.assume_init(), len.assume_init())
            };
            assert_eq!(expected_len, len);
            owner.delete(ptr);
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
        owner.delete(&x as *const i32 as *mut i32);
    }
}
