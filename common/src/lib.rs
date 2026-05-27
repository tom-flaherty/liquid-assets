#![no_std]

pub trait Decompressor {
    fn decompress(&self) -> Result<(), ()>;
}
