#[cfg(feature = "download")]
pub mod download;

#[cfg(feature = "link")]
pub mod link;

#[cfg(any(feature = "download", feature = "link"))]
pub type Error = anyhow::Error;

#[cfg(not(any(feature = "download", feature = "link")))]
pub type Error = std::convert::Infallible;
