pub(crate) use self::vec::NonEmptyVec;

impl<T> NonEmptyVec<T> {
    pub(crate) fn first(&self) -> &T {
        self.as_ref().first().expect("this should be non-empty")
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_ref().iter()
    }
}

mod vec {
    use derive_more::AsRef;

    #[derive(AsRef)]
    #[as_ref([T])]
    pub(crate) struct NonEmptyVec<T>(
        Vec<T>, // invariant: must be non-empty
    );

    impl<T> NonEmptyVec<T> {
        pub(crate) fn new(vec: Vec<T>) -> Option<Self> {
            (!vec.is_empty()).then_some(Self(vec))
        }
    }
}
