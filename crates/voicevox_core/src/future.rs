use std::future::Future;

use easy_ext::ext;

/// `futures_lite::future::block_on`を、[pollster]のように`.block_on()`という形で使えるようにする。
///
/// [pollster]: https://docs.rs/crate/pollster
#[ext(FutureExt)]
impl<F: Future> F {
    pub(crate) fn block_on(self) -> Self::Output
    where
        Self: Sized,
    {
        futures_lite::future::block_on(self)
    }
}
