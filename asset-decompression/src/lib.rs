#![no_std]

pub use my_proc_macro::include_graphics;

pub trait Decompressor {
    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<usize, ()>;
}
