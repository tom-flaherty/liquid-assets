#![no_std]

pub use liquid_assets_inflate_proc_macro::include_assets;

pub trait Decompressor {
    type Error;

    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<usize, Self::Error>;
}
