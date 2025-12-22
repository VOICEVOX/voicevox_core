use easy_ext::ext;
use typeshare::U53;

macro_rules! positive_finite_f32 {
    ($value:literal $(,)?) => {{
        const VALUE: typed_floats::PositiveFinite<f32> =
            if let Ok(value) = typed_floats::PositiveFinite::<f32>::new($value) {
                value
            } else {
                panic!("invalid")
            };
        VALUE
    }};
}

pub(crate) use positive_finite_f32;

#[ext(U53Ext)]
impl U53 {
    pub(crate) fn to_i64(self) -> i64 {
        u64::from(self).try_into().expect("this is 53-bit")
    }
}
