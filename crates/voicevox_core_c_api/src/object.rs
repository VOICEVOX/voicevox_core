#![expect(
    clippy::type_complexity,
    reason = "`CApiObject::bodies`に対するもの。型を分離するとかえって可読性を失う。その代わりコメ\
              ントを入れている。`#[…]`じゃなくて`#![…]`でやってるのは、Clippy 0.1.83でeasy-extに反\
              応するようになってしまったため"
)]
use std::{
    any,
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display},
    mem,
    num::NonZero,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::Arc,
};

use easy_ext::ext;
use tracing::warn;

// FIXME: 次のような状況に備え、`new`をいっぱい行うテストを書く
// https://github.com/VOICEVOX/voicevox_core/pull/849#discussion_r1814221605

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

    // 行う可変操作は`insert`のみ
    fn known_addrs() -> &'static std::sync::Mutex<HashSet<NonZero<usize>>>;

    fn heads() -> &'static boxcar::Vec<Self>;

    fn bodies() -> &'static std::sync::Mutex<
        HashMap<
            NonZero<usize>, // `heads`の要素へのポインタのアドレス
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
            let i = Self::heads().push(Default::default());
            NonNull::from(&Self::heads()[i])
        };
        Self::lock_known_addrs().insert(this.addr());
        let body = parking_lot::RwLock::new(body.into()).into();
        Self::lock_bodies().insert(this.addr(), body);
        this
    }
}

#[ext(CApiObjectPtrExt)]
impl<T: CApiObject> *const T {
    // ユーザーから渡されたポインタである`self`は、次のうちどれかに分類される。
    //
    // 1. `known_addrs`に含まれない ⇨ 知らないどこかのダングリングポインタか何か。あるいはnull
    // 2. `known_addrs`に含まれるが、`bodies`に含まれない → "delete"済み
    // 3. `known_addrs`も`bodies`にも含まれる → 1.でも2.でもなく、有効

    /// # Panics
    ///
    /// 同じ対象に対して`drop_body`を呼んでいるとパニックする。
    pub(crate) fn body(self) -> impl Deref<Target = T::RustApiObject> {
        let this = self.validate();

        let body = T::lock_bodies()
            .get(&this.addr())
            .unwrap_or_else(|| this.panic_for_deleted())
            .read_arc();

        voicevox_core::__internal::interop::raii::try_map_guard(body, |body| {
            body.as_ref().ok_or(())
        })
        .unwrap_or_else(|()| this.panic_for_deleted())
    }

    /// # Panics
    ///
    /// 同じ対象に対してこの関数を二度呼ぶとパニックする。
    pub(crate) fn drop_body(self) {
        if self.is_null() {
            return;
        }

        let this = self.validate();

        let body = T::lock_bodies()
            .remove(&this.addr())
            .unwrap_or_else(|| this.panic_for_deleted());

        drop(
            body.try_write_arc()
                .unwrap_or_else(|| {
                    warn!(
                        "{this} is still in use. Waiting before closing",
                        this = this.display(),
                    );
                    body.write_arc()
                })
                .take()
                .unwrap_or_else(|| this.panic_for_deleted()),
        );
    }
}

#[ext]
impl<T: CApiObject> *const T {
    fn validate(self) -> NonNull<T> {
        let this = NonNull::new(self as *mut T).expect("the argument must not be null");
        if !T::lock_known_addrs().contains(&this.addr()) {
            panic!("{self:018p} does not seem to be valid object");
        }
        this
    }
}

#[ext]
impl<T: CApiObject> NonNull<T> {
    fn display(self) -> impl Display {
        let type_name = any::type_name::<T>()
            .split("::")
            .last()
            .expect("should not empty");
        fmt::from_fn(move |f| write!(f, "`{type_name}` ({self:018p})"))
    }

    fn panic_for_deleted(self) -> ! {
        panic!("{}は既に破棄されています", self.display());
    }
}

#[ext]
impl<T: CApiObject> T {
    fn lock_known_addrs() -> impl DerefMut<Target = HashSet<NonZero<usize>>> {
        Self::known_addrs().lock().unwrap_or_else(|e| panic!("{e}"))
    }

    fn lock_bodies() -> impl DerefMut<
        Target = HashMap<NonZero<usize>, Arc<parking_lot::RwLock<Option<Self::RustApiObject>>>>,
    > {
        Self::bodies().lock().unwrap_or_else(|e| panic!("{e}"))
    }
}
