use std::{
    any,
    collections::HashMap,
    fmt::{Debug, Display},
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::Arc,
};

use easy_ext::ext;
use tracing::warn;

/// プロセスの終わりまでデストラクトされない、ユーザーにオブジェクトとして貸し出す1-bit長の構造体。
///
/// インスタンスは次のような形。
///
/// ```
/// pub struct VoicevoxSynthesizer {
///    _padding: MaybeUninit<[u8; 1]>,
/// }
/// ```
///
/// `RustApiObject`そのものではなくこのトレイトのインスタンスをユーザーに渡すようにすることで、次のことを実現する。
///
/// 1. "delete"時に対象オブジェクトに対するアクセスがあった場合、アクセスが終わるまで待つ
/// 2. 次のユーザー操作に対するセーフティネットを張り、パニックするようにする
///     1. "delete"後に他の通常のメソッド関数の利用を試みる
///     2. "delete"後に"delete"を試みる
///     3. そもそもオブジェクトとして変なダングリングポインタが渡される
pub(crate) trait CApiObject: Default + Debug + 'static {
    type RustApiObject: 'static;

    // 書き込み操作としては`push`のみ
    fn heads() -> &'static std::sync::Mutex<Vec<Self>>;

    #[expect(
        clippy::type_complexity,
        reason = "型を分離するとかえって可読性を失う。その代わりコメントを入れている"
    )]
    fn bodies() -> &'static std::sync::Mutex<
        HashMap<
            usize, // `heads`の要素へのポインタのアドレス
            Arc<
                parking_lot::RwLock<
                    Option<Self::RustApiObject>, // `RwLock`をdropする直前まで`Some`
                >,
            >,
        >,
    >;

    fn new(body: Self::RustApiObject) -> NonNull<Self> {
        assert!(mem::size_of::<Self>() > 0);

        let this = {
            let mut heads = Self::lock_heads();
            heads.push(Default::default());
            NonNull::from(heads.last().expect("just pushed"))
        };
        let body = parking_lot::RwLock::new(body.into()).into();
        Self::lock_bodies().insert(this.as_ptr() as _, body);
        this
    }
}

#[ext(CApiObjectPtrExt)]
impl<T: CApiObject> *const T {
    /// # Panics
    ///
    /// 同じ対象に対して`drop_body`を呼んでいるとパニックする。
    pub(crate) fn body(self) -> impl Deref<Target = T::RustApiObject> {
        self.validate();

        let body = T::lock_bodies()
            .get(&(self as _))
            .unwrap_or_else(|| self.panic_for_deleted())
            .read_arc();

        voicevox_core::__internal::interop::raii::try_map_guard(body, |body| {
            body.as_ref().ok_or(())
        })
        .unwrap_or_else(|()| self.panic_for_deleted())
    }

    /// # Panics
    ///
    /// 同じ対象に対してこの関数を二度呼ぶとパニックする。
    pub(crate) fn drop_body(self) {
        self.validate();

        let body = T::lock_bodies()
            .remove(&(self as _))
            .unwrap_or_else(|| self.panic_for_deleted());

        drop(
            body.try_write_arc()
                .unwrap_or_else(|| {
                    warn!(
                        "{this} is still in use. Waiting before closing",
                        this = self.display(),
                    );
                    body.write_arc()
                })
                .take()
                .unwrap_or_else(|| self.panic_for_deleted()),
        );
    }
}

#[ext]
impl<T: CApiObject> *const T {
    fn validate(self) {
        if self.is_null() {
            panic!("the argument must not be null");
        }
        if !T::lock_heads().as_ptr_range().contains(&self) {
            panic!("{self:018p} does not seem to be valid object");
        }
    }

    fn display(self) -> impl Display {
        let type_name = any::type_name::<T>()
            .split("::")
            .last()
            .expect("should not empty");
        format!("`{type_name}` ({self:018p})")
    }

    fn panic_for_deleted(self) -> ! {
        panic!("{}は既に破棄されています", self.display());
    }
}

#[ext]
impl<T: CApiObject> T {
    fn lock_heads() -> impl DerefMut<Target = Vec<Self>> {
        Self::heads().lock().unwrap_or_else(|e| panic!("{e}"))
    }

    fn lock_bodies(
    ) -> impl DerefMut<Target = HashMap<usize, Arc<parking_lot::RwLock<Option<Self::RustApiObject>>>>>
    {
        Self::bodies().lock().unwrap_or_else(|e| panic!("{e}"))
    }
}
