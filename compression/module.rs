use crate::size::Size;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::io::{self, Write};

pub trait Compression:
    Debug + Serialize + DeserializeOwned + Size + Clone + Send + Sync + 'static
{
    type Compress: Compress;
    fn decompress(&self, buffer: Box<[u8]>) -> io::Result<Box<[u8]>>;
    fn compress(&self) -> Self::Compress;
}

pub trait Compress: Write {
    fn finish(self) -> Box<[u8]>;
}

mod none;
pub use self::none::None;

mod lz4;
pub use self::lz4::Lz4;

