use easy_ext::ext;

#[ext(F32Ext)]
pub(crate) impl f32 {
    /// 偶数丸めを行う。
    ///
    /// [`round_ties_even` feature]で追加される予定の`f32::round_ties_even`の代用。
    ///
    /// [`round_ties_even` feature]: https://github.com/rust-lang/rust/pull/95317
    fn round_ties_even_(self) -> f32 {
        let mut rounded = self.round();
        if (self - rounded).abs() == 0.5 {
            rounded = 2. * (self / 2.).round();
        }
        rounded
    }
}
