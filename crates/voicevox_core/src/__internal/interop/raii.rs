use std::{marker::PhantomData, ops::Deref};

use ouroboros::self_referencing;

#[derive(Debug)]
pub enum MaybeClosed<T> {
    Open(T),
    Closed,
}

// [`mapped_lock_guards`]のようなことをやるためのユーティリティ。
//
// [`mapped_lock_guards`]: https://github.com/rust-lang/rust/issues/117108
pub fn try_map_guard<'lock, G, F, T, E>(guard: G, f: F) -> Result<impl Deref<Target = T> + 'lock, E>
where
    G: 'lock,
    F: FnOnce(&G) -> Result<&T, E>,
    T: 'lock,
{
    return MappedLockTryBuilder {
        guard,
        target_builder: f,
        marker: PhantomData,
    }
    .try_build();

    #[self_referencing]
    struct MappedLock<'lock, G: 'lock, T> {
        guard: G,

        #[borrows(guard)]
        target: &'this T,

        marker: PhantomData<&'lock T>,
    }

    impl<'lock, G: 'lock, T: 'lock> Deref for MappedLock<'lock, G, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.borrow_target()
        }
    }
}
