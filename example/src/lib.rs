#![no_std]

#[cfg(feature = "display")]
use assets::DecompressedData;
#[cfg(feature = "display")]
use esp_hal::peripherals::Peripherals;
#[cfg(feature = "display")]
use esp_hal::time::Duration;
use esp_hal::time::Instant;
use rtt_target::rprintln;

mod decompressors;
use decompressors::*;

const BUFFER_SIZE: usize = 128 * 128 * 2;

liquid_assets_inflate::include_assets!("asset-binaries", BUFFER_SIZE);
// liquid_assets_inflate::include_assets!("asset-binaries", 32768);

#[cfg(not(feature = "display"))]
pub fn run_benchmark() -> ! {
    let mut buffer = [0_u8; BUFFER_SIZE];

    let start_time = Instant::now();

    let decompressor = MinizOxideDecompressor {};

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
        spi::{Mode, master},
        time::Rate,
    };
    use mipidsi::{Builder, interface::SpiInterface};
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

    // let decompressor = MinizOxideDecompressor {};
    let decompressor = NoDecompressor {};
    // let decompressor = LzssDecompressor {};

    let mut frame_buffer = [0_u8; 135 * 240 * 2];

    let delay = Delay::new();
    let mut last_frame_drawn = Instant::now();
    let mut last_draw_duration = Duration::from_secs(0);
    loop {
        for (frame_number, frame) in assets::GITHUB.as_iter().enumerate() {
            rprint!("Frame: {} ", frame_number);

            let decompression_start = Instant::now();

            let DecompressedData {
                bytes_wrote, width, ..
            } = frame.decompress(&mut frame_buffer, &decompressor).unwrap();

            rprint!("Decompress time: {} ", decompression_start.elapsed());

            let image_raw = embedded_graphics::image::ImageRaw::<Rgb565>::new(
                &frame_buffer[0..bytes_wrote],
                width as u32,
            );
            let image = embedded_graphics::image::Image::new(&image_raw, Point { x: 0, y: 0 });

            delay.delay(
                Duration::from_millis(50)
                    .checked_sub(last_frame_drawn.elapsed())
                    .unwrap_or(Duration::from_millis(0))
                    .checked_sub(last_draw_duration)
                    .unwrap_or(Duration::from_millis(0)),
            );

            let draw_start = Instant::now();
            image.draw(&mut display).unwrap(); // TODO shouldn't use this function directly
            last_draw_duration = last_frame_drawn.elapsed();
            rprintln!(
                "Draw time: {} Frame time: {}",
                draw_start.elapsed(),
                last_draw_duration
            );

            last_frame_drawn = Instant::now();
        }
    }
}
