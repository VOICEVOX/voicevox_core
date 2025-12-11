use easy_ext::ext;
use itertools::Itertools as _;
use ndarray::Array1;

#[ext(IteratorExt)]
impl<I: Iterator> I {
    pub(crate) fn unzip_into_array1s<A, B>(self) -> (Array1<A>, Array1<B>)
    where
        Self: Iterator<Item = (A, B)>,
    {
        let (xs, ys) = self.unzip::<_, _, Vec<_>, Vec<_>>();
        (xs.into(), ys.into())
    }

    pub(crate) fn multiunzip_into_array1s<FromI>(self) -> FromI
    where
        Self: MultiUnzipIntoArray1s<FromI>,
    {
        MultiUnzipIntoArray1s::multiunzip_into_array1s(self)
    }
}

pub(crate) trait MultiUnzipIntoArray1s<FromI>: Iterator {
    fn multiunzip_into_array1s(self) -> FromI;
}

impl<I: Iterator<Item = (A, B, C)>, A, B, C>
    MultiUnzipIntoArray1s<(Array1<A>, Array1<B>, Array1<C>)> for I
{
    fn multiunzip_into_array1s(self) -> (Array1<A>, Array1<B>, Array1<C>) {
        let (xs, ys, zs) = self.multiunzip::<(Vec<_>, Vec<_>, Vec<_>)>();
        (xs.into(), ys.into(), zs.into())
    }
}
