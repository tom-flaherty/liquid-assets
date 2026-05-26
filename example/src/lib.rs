#![no_std]

use esp_hal::time::Instant;
// use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_slice_iter_to_slice;
use rtt_target::rprintln;

const BUFFER_SIZE: usize = 128 * 128 * 2;

struct Decompressor {}

const DEC: Decompressor = Decompressor {};

// TODO verify that the buffer size can be either a literal or expression
// TODO verify that it still works if you use a full path (e.g. crate::BUFFER_SIZE)
// TODO need to ensure this macro isn't called within a function
asset_decompression::include_graphics!("graphics-bin", BUFFER_SIZE);
// asset_decompression::include_graphics!("graphics-bin", 32768);

pub fn run() {
    let mut frame_buffer = [0_u8; BUFFER_SIZE];

    // let compressed_bytes = include_bytes!("../../assets/output/espressif.bin").as_slice();

    let start_time = Instant::now();

    let _bytes_wrote = decompress_slice_iter_to_slice(
        &mut frame_buffer,
        core::iter::once(assets::espressif.bytes),
        false,
        false,
    )
    .unwrap();

    // let frame = assets::loading.get_frame(0, &mut frame_buffer).unwrap();
    for frame in assets::loading.as_iter() {
        rprintln!("{:?}", frame);
    }

    let duration = start_time.elapsed();

    rprintln!("Decompression took {:?}", duration);
}
