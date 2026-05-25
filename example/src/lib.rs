#![no_std]

use esp_hal::time::Instant;
// use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_slice_iter_to_slice;
use rtt_target::rprintln;

asset_decompression::include_graphics!("graphics-bin");

pub fn run() {
    let mut frame_buffer = [0_u8; 128 * 128 * 2];

    // let compressed_bytes = include_bytes!("../../assets/output/espressif.bin").as_slice();

    let start_time = Instant::now();

    let _bytes_wrote = decompress_slice_iter_to_slice(
        &mut frame_buffer,
        core::iter::once(assets::espressif.bytes),
        false,
        false,
    )
    .unwrap();

    let duration = start_time.elapsed();

    rprintln!("Decompression took {:?}", duration);
}
