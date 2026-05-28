/// Expected structure for input directory:
///
/// input
/// ├── animation_name1
/// │   ├── frame1.png
/// │   ├── frame2.png
/// │   ├── frame3.png
/// │   ├── frame4.png
/// │   └── ...
/// ├── animation_name2
/// │   ├── frame1.png
/// │   ├── frame2.png
/// │   ├── frame3.png
/// │   ├── frame4.png
/// │   └── ...
/// ├── asset_name1.png
/// ├── asset_name2.png
/// └── asset_name3.png
///
/// Non-png files can be included, e.g. notes or a source gif. These will be ignored.
///
mod dir;
mod processing;

use dir::prepare_output_directory;

use crate::processing::AssetProcessor;
pub use crate::processing::TargetColorFormat;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

pub trait Compressor {
    type Error;

    fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error>;
}

pub fn rebuild_graphics_if_changed<C: Compressor>(
    input_dir: &'static str,
    output_dir: &'static str,
    target_color_format: TargetColorFormat,
    compressor: C,
) where
    <C as Compressor>::Error: fmt::Debug,
{
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let input_path = PathBuf::from(&cargo_manifest_dir).join(Path::new(input_dir));

    println!(
        "Assets source path is {}",
        input_path.as_path().to_str().unwrap()
    );

    if !input_path.try_exists().unwrap_or(false) {
        panic!(
            "Input directory does not exist {}",
            input_path.as_path().to_str().unwrap()
        );
    }

    println!("cargo:rerun-if-changed={}", input_path.to_str().unwrap());
    println!("cargo:rerun-if-env-changed=REBUILD_ASSETS");

    let output_path = PathBuf::from(&cargo_manifest_dir).join(Path::new(output_dir));

    println!("Assets output path is {}", output_path.to_str().unwrap());

    prepare_output_directory(output_path.as_path());

    if !output_path.try_exists().unwrap_or(false) {
        panic!("Failed to create output directory");
    }

    let mut asset_processor = AssetProcessor::new(target_color_format);

    asset_processor.process(input_path.as_path(), output_path.as_path(), &compressor);

    let stats: String = asset_processor.generate_stats();
    println!("{}", stats);
    fs::write(
        output_path.join("statistics.txt").as_path(),
        stats.as_bytes(),
    )
    .unwrap();
}

// To view output logs when running the test, run `cargo test -- --nocapture`
#[cfg(test)]
mod tests {
    use super::{Compressor, TargetColorFormat, rebuild_graphics_if_changed};

    struct ZlibCompressor {}
    impl Compressor for ZlibCompressor {
        type Error = ();

        fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
            const COMPRESSION_LEVEL: u8 = 5;
            Ok(miniz_oxide::deflate::compress_to_vec(
                input_bytes,
                COMPRESSION_LEVEL,
            ))
        }
    }

    #[test]
    fn check_decompression() {
        let compressor = ZlibCompressor {};
        rebuild_graphics_if_changed(
            "test_input",
            "test_output",
            TargetColorFormat::Rgb565,
            compressor,
        )
    }
}
