#![no_std]

use asset_decompression::Decompressor;
use esp_hal::time::Instant;
use rtt_target::rprintln;

const BUFFER_SIZE: usize = 128 * 128 * 2;

struct ZlibDecompressor {}
impl Decompressor for ZlibDecompressor {
    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<(), ()> {
        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            buffer,
            core::iter::once(compressed_data),
            false,
            false,
        )
        .map_err(|_e| ())?;
        Ok(())
    }
}

// const DEC: Decompressor = Decompressor {};

// TODO verify that the buffer size can be either a literal or expression
// TODO verify that it still works if you use a full path (e.g. crate::BUFFER_SIZE)
// TODO need to ensure this macro isn't called within a function
asset_decompression::include_graphics!("graphics-bin", BUFFER_SIZE);
// asset_decompression::include_graphics!("graphics-bin", 32768);

pub fn run() {
    let mut buffer = [0_u8; BUFFER_SIZE];

    let start_time = Instant::now();

    let decompressor = ZlibDecompressor {};

    assets::ESPRESSIF
        .decompress(&mut buffer, &decompressor)
        .unwrap();

    // let _bytes_wrote = decompress_slice_iter_to_slice(
    //     &mut frame_buffer,
    //     core::iter::once(assets::ESPRESSIF.bytes),
    //     false,
    //     false,
    // )
    // .unwrap();

    // for frame in assets::LOADING.as_iter() {
    //     rprintln!("{:?}", frame[0]);
    // }

    let duration = start_time.elapsed();

    rprintln!("Decompression took {:?}", duration);
}
