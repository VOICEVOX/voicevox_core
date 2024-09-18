use std::{marker::PhantomData, ops::Deref};

use ouroboros::self_referencing;

pub enum MaybeClosed<T> {
    Open(T),
    Closed,
}

// [`mapped_lock_guards`]のようなことをやるためのユーティリティ。
//
// `T: 'static`が入っているのは、`T` outlive `G`の関係にすることによりouroborosに突っ込んでSafe
// Rustで記述しきるため。
//
// [`mapped_lock_guards`]: https://github.com/rust-lang/rust/issues/117108
pub fn try_map_guard<'a, G, F, T, E>(guard: G, f: F) -> Result<impl Deref<Target = T> + 'a, E>
where
    G: 'a,
    F: FnOnce(&G) -> Result<&T, E>,
    T: 'static,
{
    return MappedLockTryBuilder {
        guard,
        content_builder: f,
        marker: PhantomData,
    }
    .try_build();

    #[self_referencing]
    struct MappedLock<'a, G: 'a, T: 'static> {
        guard: G,

        #[borrows(guard)]
        content: &'this T,

        marker: PhantomData<&'a ()>,
    }

    impl<'a, G: 'a, T: 'static> Deref for MappedLock<'a, G, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.borrow_content()
        }
    }
}
