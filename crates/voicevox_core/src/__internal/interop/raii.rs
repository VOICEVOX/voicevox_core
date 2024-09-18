use std::{marker::PhantomData, ops::Deref};

use ouroboros::self_referencing;

pub enum MaybeClosed<T> {
    Open(T),
    Closed,
}

// [`mapped_lock_guards`]のようなことをやるためのユーティリティ。
//
// [`mapped_lock_guards`]: https://github.com/rust-lang/rust/issues/117108
pub fn try_map_guard<'lock, 'target, G, F, T, E>(
    guard: G,
    f: F,
) -> Result<impl Deref<Target = T> + 'lock, E>
where
    'target: 'lock,
    G: 'lock,
    F: FnOnce(&G) -> Result<&T, E>,
    T: 'target,
{
    return MappedLockTryBuilder {
        guard,
        content_builder: f,
        marker: PhantomData,
    }
    .try_build();

    #[self_referencing]
    struct MappedLock<'lock, 'target, G: 'lock, T: 'target> {
        guard: G,

        #[borrows(guard)]
        content: &'this T,

        marker: PhantomData<&'lock &'target ()>,
    }

    impl<'lock, 'target: 'lock, G: 'lock, T: 'target> Deref for MappedLock<'lock, 'target, G, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.borrow_content()
        }
    }
}
