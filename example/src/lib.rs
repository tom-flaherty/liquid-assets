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
    ) -> Result<usize, ()> {
        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            buffer,
            core::iter::once(compressed_data),
            false,
            false,
        )
        .map_err(|_e| ())
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

    // Decompress a single static asset
    assets::ESPRESSIF
        .decompress(&mut buffer, &decompressor)
        .unwrap();

    rprintln!("Decompression took {:?}", start_time.elapsed());

    // Get the number of frames in an animation
    let loading_frames = assets::LOADING.get_number_of_frames();
    rprintln!("The loading animation contains {} frames", loading_frames);

    // Decompress a single frame by passing a reference to the decompressor
    let start_time = Instant::now();
    let frame_number = 3;
    let bytes_written = assets::LOADING
        .decompress_frame(frame_number, &mut buffer, &decompressor)
        .unwrap();
    rprintln!(
        "Decompressed frame {} of loading animation. Wrote {} bytes in {}",
        frame_number,
        bytes_written,
        start_time.elapsed()
    );

    // Decompress a single frame by getting the raw data and decompressing using
    // the library directly
    let frame_number = 5;
    let data = assets::LOADING
        .get_compressed_frame_data(frame_number)
        .unwrap();
    miniz_oxide::inflate::decompress_slice_iter_to_slice(
        &mut buffer,
        core::iter::once(data),
        false,
        false,
    )
    .unwrap();

    // assets::LOADING.copy_compressed_frame_data_to_buffer();

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
