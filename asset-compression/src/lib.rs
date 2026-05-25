use std::path::Path;
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

#[derive(Debug)]
pub enum CompError {
    CannotVerifyInputDirExists,
    CannotVerifyOutputDirExists,
}

// TODO it should find all the assets first so that it can print progress, e.g. 22/300

pub fn rebuild_graphics_if_changed(
    input_dir: &'static str,
    output_dir: &'static str,
    target_color_format: TargetColorFormat,
) -> Result<(), CompError> {
    let cargo_manifest_str = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let input_dir_str = format!("{}/{}", cargo_manifest_str, input_dir);
    let input_dir = Path::new(&input_dir_str);

    println!("input directory full path is {}", input_dir_str);

    if !input_dir
        .try_exists()
        .map_err(|_| CompError::CannotVerifyInputDirExists)?
    {
        return Err(CompError::CannotVerifyInputDirExists);
    }

    // Tell cargo to rerun the build script whenever this folder has changed
    println!("cargo:rerun-if-changed={}", input_dir.to_str().unwrap());

    let output_dir_str = format!("{}/{}", cargo_manifest_str, output_dir);
    let output_dir = Path::new(&output_dir_str);

    println!("output directory full path is {}", output_dir_str);

    // Ensure output directory is empty
    prepare_output_directory(output_dir);

    if !output_dir
        .try_exists()
        .map_err(|_| CompError::CannotVerifyOutputDirExists)?
    {
        return Err(CompError::CannotVerifyOutputDirExists);
    }

    let mut asset_processor = AssetProcessor::new(target_color_format);
    asset_processor.process(input_dir, output_dir);
    asset_processor.print_stats();

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use image::EncodableLayout;

//     #[test]
//     fn check_decompression() {
//         use image::ImageReader;
//         let expected_vec = {
//             let image = ImageReader::open("./input/espressif.png")
//                 .unwrap()
//                 .decode()
//                 .unwrap();
//             let mut pixels565: Vec<u16> = Vec::new();
//             for pixel in image.to_rgb8().pixels() {
//                 pixels565.push(super::rgb888torgb565(pixel[0], pixel[1], pixel[2]));
//             }
//             pixels565
//         };
//         let expected_bytes = expected_vec.as_bytes();

//         let compressed_bytes = include_bytes!("../output/espressif.bin").as_bytes();
//         let mut output_bytes = [0_u8; 128 * 128 * 2];
//         let _bytes_wrote = miniz_oxide::inflate::decompress_slice_iter_to_slice(
//             &mut output_bytes,
//             core::iter::once(compressed_bytes),
//             false,
//             false,
//         )
//         .unwrap();

//         assert_eq!(expected_bytes.len(), 128 * 128 * 2);

//         assert_eq!(output_bytes, expected_bytes);
//     }
// }
