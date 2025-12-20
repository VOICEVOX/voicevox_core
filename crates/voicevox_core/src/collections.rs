use std::{iter::Sum, num::NonZero, ops::Deref};

use self::slice_iter::NonEmptySliceIter;

pub(crate) use self::{slice::NonEmptySlice, vec::NonEmptyVec};

pub(crate) trait NonEmptyIterator: AssertNonEmpty {
    fn map<B, F>(self, f: F) -> impl NonEmptyIterator<Item = B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        return Map(self.into_iter().map(f));

        struct Map<I>(I);

        impl<I: Iterator> AssertNonEmpty for Map<I> {}

        impl<I: Iterator> IntoIterator for Map<I> {
            type Item = I::Item;
            type IntoIter = I;

            fn into_iter(self) -> Self::IntoIter {
                self.0
            }
        }
    }

    fn collect<B>(self) -> B
    where
        B: FromNonEmptyIterator<Self::Item>,
        Self: Sized,
    {
        FromNonEmptyIterator::from_non_empty_iter(self)
    }

    fn sum<S>(self) -> S
    where
        S: Sum<Self::Item>,
        Self: Sized,
    {
        self.into_iter().sum()
    }
}

impl<I: AssertNonEmpty> NonEmptyIterator for I {}

/// # Invariant
///
/// The `IntoIter` must be non-empty.
pub(crate) trait AssertNonEmpty: IntoIterator {}

pub(crate) trait FromNonEmptyIterator<A>: Sized {
    fn from_non_empty_iter<T>(iter: T) -> Self
    where
        T: IntoNonEmptyIterator<Item = A>;
}

impl<A> FromNonEmptyIterator<A> for Vec<A> {
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = A>,
    {
        iter.into_iter().collect()
    }
}

pub(crate) trait IntoNonEmptyIterator: IntoIterator {
    fn _into_non_empty_iter(self) -> impl NonEmptyIterator<Item = Self::Item>;
}

impl<I: NonEmptyIterator> IntoNonEmptyIterator for I {
    fn _into_non_empty_iter(self) -> impl NonEmptyIterator<Item = Self::Item> {
        self
    }
}

impl<T> NonEmptySlice<T> {
    pub(crate) fn len(&self) -> NonZero<usize> {
        NonZero::new(self.as_ref().len()).expect("this should be non-empty")
    }

    pub(crate) fn first(&self) -> &T {
        self.as_ref().first().expect("this should be non-empty")
    }

    pub(crate) fn split_first(&self) -> (&T, &[T]) {
        self.as_ref()
            .split_first()
            .expect("this should be non-empty")
    }

    pub(crate) fn split_last(&self) -> (&T, &[T]) {
        self.as_ref()
            .split_last()
            .expect("this should be non-empty")
    }

    pub(crate) fn iter(&self) -> impl NonEmptyIterator<Item = &T> {
        NonEmptySliceIter::new(self.as_ref().iter()).expect("should have the same invariant")
    }
}

impl<'a, T> IntoIterator for &'a NonEmptySlice<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

impl<T> FromNonEmptyIterator<T> for NonEmptyVec<T> {
    fn from_non_empty_iter<I>(iter: I) -> Self
    where
        I: IntoNonEmptyIterator<Item = T>,
    {
        let inner = FromNonEmptyIterator::from_non_empty_iter(iter);
        Self::new(inner).expect("should have the same invariant")
    }
}

impl<T> Deref for NonEmptyVec<T> {
    type Target = NonEmptySlice<T>;

    fn deref(&self) -> &Self::Target {
        NonEmptySlice::new(self.as_ref()).expect("should have the same invariant")
    }
}

mod slice {
    use derive_more::AsRef;
    use ref_cast::{ref_cast_custom, RefCastCustom};

    #[derive(RefCastCustom, AsRef)]
    #[repr(transparent)]
    pub(crate) struct NonEmptySlice<T>(
        /// # Invariant
        ///
        /// This must be non-empty.
        [T],
    );

    impl<T> NonEmptySlice<T> {
        pub(crate) fn new(slice: &[T]) -> Option<&Self> {
            (!slice.is_empty()).then(|| Self::new_(slice))
        }

        #[ref_cast_custom]
        fn new_(slice: &[T]) -> &Self;
    }
}

mod slice_iter {
    use std::slice;

    use super::AssertNonEmpty;

    pub(crate) struct NonEmptySliceIter<'a, T>(
        /// # Invariant
        ///
        /// This must be non-empty.
        slice::Iter<'a, T>,
    );

    impl<'a, T> NonEmptySliceIter<'a, T> {
        pub(super) fn new(iter: slice::Iter<'a, T>) -> Option<Self> {
            (iter.len() > 0).then_some(Self(iter))
        }
    }

    impl<'a, T: 'a> AssertNonEmpty for NonEmptySliceIter<'a, T> {}

    impl<'a, T: 'a> IntoIterator for NonEmptySliceIter<'a, T> {
        type Item = &'a T;
        type IntoIter = slice::Iter<'a, T>;

        fn into_iter(self) -> Self::IntoIter {
            self.0
        }
    }
}

mod vec {
    use derive_more::AsRef;

    #[derive(AsRef)]
    #[as_ref([T])]
    pub(crate) struct NonEmptyVec<T>(
        /// # Invariant
        ///
        /// This must be non-empty.
        Vec<T>,
    );

    impl<T> NonEmptyVec<T> {
        pub(crate) fn new(vec: Vec<T>) -> Option<Self> {
            (!vec.is_empty()).then_some(Self(vec))
        }
    }
}
