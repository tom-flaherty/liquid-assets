#![no_std]

#[cfg(feature = "add_display")]
use esp_hal::time::Instant;
#[cfg(feature = "add_display")]
use esp_hal::{
    peripherals::{self, Peripherals},
    time::Rate,
};
use liquid_assets_inflate::Decompressor;
use rtt_target::rprintln;

#[cfg(feature = "add_display")]
mod ssd1327;

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

    // let duration = start_time.elapsed();

    // rprintln!("Decompression took {:?}", duration);
}

#[cfg(feature = "add_display")]
pub fn run_display_loop(peripherals: Peripherals) -> ! {
    use embedded_hal::spi::SpiBus;
    use esp_hal::{
        delay::Delay,
        gpio,
        spi::{Mode, master},
        time::Rate,
    };

    rprintln!("Setting up display");

    let cs = gpio::Output::new(
        peripherals.GPIO5,
        gpio::Level::High,
        gpio::OutputConfig::default(),
    );

    let spi_config = master::Config::default()
        .with_mode(Mode::_3)
        .with_frequency(Rate::from_mhz(1));
    let spi = master::Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_mosi(peripherals.GPIO2)
        .with_sck(peripherals.GPIO4)
        .with_cs(cs);

    let dc = gpio::Output::new(
        peripherals.GPIO6,
        gpio::Level::Low,
        gpio::OutputConfig::default(),
    );
    let _rst = gpio::Output::new(
        peripherals.GPIO7,
        gpio::Level::Low,
        gpio::OutputConfig::default(),
    );

    let mut display = ssd1327::Ssd1327::new(spi, dc);
    display.init();
    display.clear();

    // let display = ssd1327::display::Ssd1327::new(spi_interface);

    // // esp_hal::spi::master::
    // let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    // let spi_interface = SpiInterface::new(spi_device, dc, &mut buffer);

    // let mut buffer = [0_u8; 128 * 128 * 2];

    // let delay = Delay::new();

    // let mut display = Builder::new(mipidsi::models::, di)
    //     .reset_pin(rst)
    //     .init(&mut delay)
    //     .unwrap();

    loop {}
}
