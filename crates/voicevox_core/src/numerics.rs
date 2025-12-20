use easy_ext::ext;
use typeshare::U53;

#[ext(U53Ext)]
impl U53 {
    pub(crate) fn to_i64(self) -> i64 {
        u64::from(self).try_into().expect("this is 53-bit")
    }
}
