use easy_ext::ext;
use typeshare::U53;

#[cfg(test)]
macro_rules! non_zero {
    ($value:literal $(,)?) => {{
        const VALUE: std::num::NonZero<::macros::int_type!($value)> =
            if let Some(value) = std::num::NonZero::new($value) {
                value
            } else {
                panic!("invalid")
            };
        VALUE
    }};
}

#[cfg(test)]
pub(crate) use non_zero;

macro_rules! non_nan_finite_f32 {
    ($value:literal $(,)?) => {{
        const VALUE: typed_floats::NonNaNFinite<f32> =
            if let Ok(value) = typed_floats::NonNaNFinite::<f32>::new($value) {
                value
            } else {
                panic!("invalid")
            };
        VALUE
    }};
}

pub(crate) use non_nan_finite_f32;

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
