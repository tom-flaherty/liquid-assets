#![no_std]

use assets::DecompressedData;
#[cfg(feature = "display")]
use esp_hal::peripherals::Peripherals;
#[cfg(feature = "display")]
use esp_hal::time::Duration;
use esp_hal::time::Instant;
use rtt_target::rprintln;

mod decompressors;
use decompressors::*;

// Size of the buffer which image data will be decompressed into
const BUFFER_SIZE: usize = 135 * 135 * 2;

liquid_assets_inflate::include_assets!("asset-binaries", BUFFER_SIZE);
// liquid_assets_inflate::include_assets!("asset-binaries", 32768);

#[cfg(not(feature = "display"))]
pub fn run_benchmark() -> ! {
    let mut buffer = [0_u8; BUFFER_SIZE];

    // Create a decompressor - make sure it matches the compressor used in build.rs!

    let decompressor = MinizOxideDecompressor {};
    // let decompressor = LzssDecompressor::new();
    // let decompressor = NoDecompressor{};

    let start_time = Instant::now();

    // Decompress a single static asset
    let DecompressedData {
        bytes_wrote: _,
        width: _,
        height: _,
    } = assets::ESPRESSIF
        .decompress(&mut buffer, &decompressor)
        .unwrap();
    // Now you can access the decompressed data as a slice with buffer[..bytes_wrote]

    rprintln!("Decompression took {:?}", start_time.elapsed());

    // Get the number of frames in an animation
    let loading_frames = assets::LOADING.get_number_of_frames();
    rprintln!("The loading animation contains {} frames", loading_frames);

    // Decompress a single frame
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

#[cfg(feature = "display")]
pub fn run_display_loop(peripherals: Peripherals) -> ! {
    // This example is for the following board and display:
    // https://www.espboards.dev/esp32/esp32-c3-devkit-rust-1/
    // https://shop.pimoroni.com/products/adafruit-1-14-240x135-color-newxie-tft-display-st7789?variant=55022898872699
    // Wiring diagram:
    //
    //  Display   | C3 Devkit
    // -----------|---------------
    //  V+        | 3V3
    //  GND       | GND
    //  CL        | IO0
    //  DA        | IO4
    //  CS        | IO5
    //  DC        | IO1
    //  BL        | Not Connected

    use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
    use esp_hal::{
        delay::Delay,
        gpio,
        spi::{master, Mode},
        time::Rate,
    };
    use mipidsi::{interface::SpiInterface, Builder};
    use rtt_target::rprint;

    rprintln!("Setting up display");

    // Setup SPI with esp-hal
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
        .with_frequency(Rate::from_mhz(65));
    let spi = master::Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_mosi(peripherals.GPIO4)
        .with_sck(peripherals.GPIO0);
    let mut delay = Delay::new();
    let mut internal_buffer = [0_u8; 512];
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let display_interface = SpiInterface::new(spi_device, dc, &mut internal_buffer);
    // Initialise the display driver. Here we use mpipdsi for a ST7789 display
    let mut display = Builder::new(mipidsi::models::ST7789, display_interface)
        .display_size(135, 240)
        .display_offset(52, 40)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // Create a decompressor - make sure it matches the compressor used in build.rs!

    let decompressor = MinizOxideDecompressor {};
    // let decompressor = LzssDecompressor {};
    // let decompressor = NoDecompressor {};

    let mut frame_buffer = [0_u8; 135 * 240 * 2];

    let delay = Delay::new();
    // 50ms per frame (20 fps)
    let desired_frame_time = Duration::from_millis(50);
    // Loop through the animation indefinitely
    loop {
        for (frame_number, frame) in assets::GITHUB.as_iter().enumerate() {
            let frame_start_time = Instant::now();
            rprint!("Frame no. {} ", frame_number);

            let decompression_start_time = Instant::now();
            let DecompressedData {
                bytes_wrote, width, ..
            } = frame.decompress(&mut frame_buffer, &decompressor).unwrap();
            rprint!("Decomp. in {} ", decompression_start_time.elapsed());

            // Now it's up to the user to display the decompressed data
            // The mipidsi driver used in this example is compatible with embedded_graphics::Image

            let image_raw = embedded_graphics::image::ImageRaw::<Rgb565>::new(
                &frame_buffer[0..bytes_wrote],
                width as u32,
            );
            let image = embedded_graphics::image::Image::new(&image_raw, Point { x: 0, y: 0 });

            let draw_start = Instant::now();
            image.draw(&mut display).unwrap();
            rprint!("Draw time {} ", draw_start.elapsed());

            // Delay to maintain framerate. Note that decompression time may vary per frame
            delay.delay(
                desired_frame_time
                    .checked_sub(frame_start_time.elapsed())
                    .unwrap_or(Duration::from_millis(0)),
            );

            rprintln!("Frame Time {}", frame_start_time.elapsed());
        }
    }
}
