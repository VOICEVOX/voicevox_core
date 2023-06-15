//! `Box<[u8]>`や`CString`といったバッファの所有者(owner)。
//!
//! 本クレートが提供するAPIとして、バイト列/文字列の生成(create)とその解放(free)がある。
//! APIとしては"生成"時に`Box<[u8]>`/`CString`のownershipがC側に渡され、"解放"時にはそのownershipがRust側に返されるといった形となる。
//!
//! しかし実装としては`Box<impl Sized>`の場合とは異なり、何かしらの情報をRust側で保持し続けなくてはならない。
//! 実態としてはRust側がバッファの所有者(owner)であり続け、C側にはその参照が渡される形になる。
//!
//! 本モジュールはそのバッファの所有者を提供する。

mod c_string;
mod slice;

pub(crate) use self::{c_string::C_STRING_OWNER, slice::U8_SLICE_OWNER};
