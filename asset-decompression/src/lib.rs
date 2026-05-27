#![no_std]

pub use my_proc_macro::include_graphics;

// Expose the Decompressor trait as the user will need to implement it
// pub use common::Decompressor;
pub trait Decompressor {
    fn decompress(&self) -> Result<(), ()>;
}
