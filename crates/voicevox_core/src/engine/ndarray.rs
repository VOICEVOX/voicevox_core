use std::iter;

use easy_ext::ext;
use ndarray::{Array1, ArrayBase, Data, Ix1};

// TODO: ndarrayをv0.17に上げたときは`ArrayRef1Ext`のようになるべき
#[ext(ArrayBase1Ext)]
impl<S: Data<Elem = A>, A: Clone> ArrayBase<S, Ix1> {
    pub(crate) fn repeat(&self, ns: impl IntoIterator<Item = usize>) -> Array1<A> {
        itertools::zip_eq(self, ns)
            .flat_map(|(x, n)| iter::repeat_n(x.clone(), n))
            .collect()
    }
}
