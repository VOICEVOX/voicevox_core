//! `Box<[u8]>`や`CString`といったバッファの所有者(owner)。
//!
//! libcのmallocで追加のアロケーションを行うことなく、バッファを直接Cの世界に貸し出すことができる。

mod c_string;
mod slice;

pub(crate) use self::{c_string::C_STRING_OWNER, slice::U8_SLICE_OWNER};
