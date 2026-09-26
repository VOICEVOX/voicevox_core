macro_rules! assert_send_sync {
    (for<$tyarg:ident : ..> $target:ty) => {
        const _: () = {
            #[expect(dead_code)]
            fn assert_send_sync<T: Send + Sync>() {}
            fn _forall_ty_arg<$tyarg: Send + Sync>() {
                assert_send_sync::<$target>();
            }
        };
    };
    ($target:ty) => {
        const _: () = {
            const fn assert_send_sync<T: Send + Sync>() {}
            assert_send_sync::<$target>();
        };
    };
}

pub(crate) use assert_send_sync;
