#![no_std]

#[cfg(feature = "add_display")]
use esp_hal::time::{Duration, Instant};
#[cfg(feature = "add_display")]
use esp_hal::{
    peripherals::{self, Peripherals},
    time::Rate,
};
use liquid_assets_inflate::Decompressor;
use rtt_target::rprintln;

const BUFFER_SIZE: usize = 128 * 128 * 2;

struct ZlibDecompressor {}
impl Decompressor for ZlibDecompressor {
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

liquid_assets_inflate::include_assets!("asset-binaries", BUFFER_SIZE);
// liquid_assets_inflate::include_assets!("asset-binaries", 32768);

#[cfg(not(feature = "add_display"))]
pub fn run_benchmark() -> ! {
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

    // Decompress all frames in an animation using the frame iterator
    for (frame_index, frame) in assets::GITHUB.as_iter().enumerate() {
        let start_time = Instant::now();
        frame.decompress(&mut buffer, &decompressor).unwrap();
        rprintln!(
            "Decompressed frame {} in {}",
            frame_index + 1,
            start_time.elapsed()
        )
    }

    loop {}
}

#[cfg(feature = "add_display")]
pub fn run_display_loop(peripherals: Peripherals) -> ! {
    use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
    use esp_hal::{
        delay::Delay,
        gpio,
        spi::{Mode, master},
        time::Rate,
    };
    use mipidsi::{Builder, interface::SpiInterface};

    rprintln!("Setting up display");

    let cs = gpio::Output::new(
        peripherals.GPIO5,
        gpio::Level::Low,
        gpio::OutputConfig::default(),
    );
    let dc = gpio::Output::new(
        peripherals.GPIO1,
        gpio::Level::Low,
        gpio::OutputConfig::default(),
    );
    let rst = gpio::Output::new(
        peripherals.GPIO3,
        gpio::Level::Low,
        gpio::OutputConfig::default(),
    );

    let spi_config = master::Config::default()
        .with_mode(Mode::_0)
        .with_frequency(Rate::from_mhz(60));
    let spi = master::Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_mosi(peripherals.GPIO4)
        .with_sck(peripherals.GPIO0);
    let mut delay = Delay::new();
    let mut internal_buffer = [0_u8; 512];
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let display_interface = SpiInterface::new(spi_device, dc, &mut internal_buffer);

    let mut display = Builder::new(mipidsi::models::ST7789, display_interface)
        .display_size(135, 240)
        .display_offset(52, 40)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let decompressor = ZlibDecompressor {};

    let mut frame_buffer = [0_u8; 135 * 240 * 2];

    let delay = Delay::new();
    loop {
        for (frame_number, frame) in assets::GITHUB.as_iter().enumerate() {
            let frame_start = Instant::now();

            let data_size = frame.decompress(&mut frame_buffer, &decompressor).unwrap();

            let decompression_time = frame_start.elapsed();

            let image_raw =
                embedded_graphics::image::ImageRaw::<Rgb565>::new(&frame_buffer[0..data_size], 135);

            // Decompression takes an unpredictable amount of time, so it's recommended to delay between
            // decompression and displaying
            delay.delay(
                Duration::from_millis(50)
                    .checked_sub(frame_start.elapsed())
                    .unwrap_or(Duration::from_millis(0)),
            );

            rprintln!(
                "Drawing frame {}: Frame time: {} Decompression time: {}",
                frame_number,
                frame_start.elapsed(),
                decompression_time
            );

            image_raw.draw(&mut display).unwrap();
        }
    }
}
