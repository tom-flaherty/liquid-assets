use liquid_assets_inflate::Decompressor;
use lzss::LzssError;

#[allow(unused)]
pub struct MinizOxideDecompressor {}
impl Decompressor for MinizOxideDecompressor {
    type Error = miniz_oxide::inflate::TINFLStatus;

    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<usize, Self::Error> {
        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            buffer,
            core::iter::once(compressed_data),
            false,
            false,
        )
    }
}

// pub struct BrotliDecompressor {}
// impl Decompressor for BrotliDecompressor {
//     type Error;

//     fn decompress<const N: usize>(
//         &self,
//         buffer: &mut [u8; N],
//         compressed_data: &[u8],
//     ) -> Result<usize, Self::Error> {
//     }
// }

// pub struct BrotlicDecompressor {}
// impl Decompressor for BrotlicDecompressor {
//     type Error = ();

//     fn decompress<const N: usize>(
//         &self,
//         buffer: &mut [u8; N],
//         compressed_data: &[u8],
//     ) -> Result<usize, Self::Error> {
//         let mut decompressor = brotlic::DecompressorReader::new(compressed_data);
//         let mut decoded_output: heapless::Vec<u8, N> = heapless::Vec::new();
//         // decompressor.read_to_end();

//         Err(())
//     }
// }

// #[allow(unused)]
// pub struct Lz4FlexDecompressor {}
// impl Decompressor for Lz4FlexDecompressor {
//     type Error = lz4_flex::block::DecompressError;

//     fn decompress<const N: usize>(
//         &self,
//         buffer: &mut [u8; N],
//         compressed_data: &[u8],
//     ) -> Result<usize, Self::Error> {
//         let mut other_buffer = [0_u8; 135 * 135 * 2];
//         let bytes_written_maybe =
//             lz4_flex::decompress_into(compressed_data, other_buffer.as_mut_slice())?;
//         buffer.copy_from_slice(other_buffer.as_slice());
//         Ok(bytes_written_maybe)

//         // let output = lz4_flex::decompress_size_prepended(compressed_data)?;
//         // let bytes_written = output.len();
//         // buffer.copy_from_slice(output.as_slice());
//         // Ok(bytes_written)
//     }
// }

#[allow(unused)]
pub struct LzssDecompressor {}
impl Decompressor for LzssDecompressor {
    type Error = LzssError<void::Void, lzss::SliceWriteError>;

    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<usize, Self::Error> {
        use lzss::{Lzss, SliceReader, SliceWriter};
        type LzssDecoder = Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;
        LzssDecoder::decompress_stack(
            SliceReader::new(compressed_data),
            SliceWriter::new(buffer.as_mut_slice()),
        )
    }
}

#[allow(unused)]
pub struct NoDecompressor {}
impl Decompressor for NoDecompressor {
    type Error = ();

    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<usize, Self::Error> {
        buffer[..compressed_data.len()].copy_from_slice(compressed_data);
        Ok(compressed_data.len())
    }
}