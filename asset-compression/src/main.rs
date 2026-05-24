use image::{EncodableLayout, ImageReader};
use log::error;
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
use std::{fs, path::Path, process::exit};

struct Args {
    assets_input_dir: String,
}

fn main() {
    env_logger::init();
    let Args { assets_input_dir } = process_args(std::env::args().collect());

    let assets_input_dir = Path::new(&assets_input_dir);

    let mut assets_output_dir = assets_input_dir.parent().unwrap().to_path_buf();
    assets_output_dir.push("output");
    let assets_output_dir = Path::new(&assets_output_dir);

    prepare_output_directory(assets_output_dir);

    process_assets(assets_input_dir, assets_output_dir);
}

fn process_args(args: Vec<String>) -> Args {
    if args.len() <= 1 {
        error!("Path to graphics input must be provided. e.g. cargo run -- ../path/to/graphics");
        exit(1);
    } else if args.len() >= 3 {
        error!("Too many arguments provided");
        exit(1);
    } else {
        Args {
            assets_input_dir: args[1].clone(),
        }
    }
}

fn prepare_output_directory(output_dir: &Path) {
    if fs::exists(output_dir).unwrap() {
        // Delete the existing directory including its contents
        fs::remove_dir_all(output_dir).unwrap()
    }
    fs::create_dir(output_dir).unwrap()
}

fn process_assets(input_path: &Path, output_dir: &Path) {
    for asset in fs::read_dir(input_path).unwrap() {
        let asset = asset.unwrap();
        let file_type = asset.file_type().unwrap();
        if file_type.is_file() {
            process_static_asset(Path::new(&asset.path()), output_dir);
        } else if file_type.is_dir() {
            process_animated_asset(Path::new(&asset.path()));
        } else {
            panic!()
        }
    }
}

fn process_static_asset(static_asset_path: &Path, output_dir: &Path) {
    // Load the image
    let image = ImageReader::open(static_asset_path)
        .unwrap()
        .decode()
        .unwrap();
    let width = image.width();
    let height = image.height();
    // Convert the image to rgb565
    let mut pixels565: Vec<u16> = Vec::new();
    for pixel in image.to_rgb8().pixels() {
        pixels565.push(rgb888torgb565(pixel[0], pixel[1], pixel[2]));
    }
    assert_eq!(pixels565.len(), (width * height) as usize);
    // Compress
    let compressed_data = miniz_oxide::deflate::compress_to_vec(pixels565.as_bytes(), 10);

    let output_filename = static_asset_path.with_extension("bin");
    let output_filename = output_filename.file_name().unwrap();
    let mut output_path = output_dir.to_path_buf();
    output_path.push(output_filename);
    fs::write(output_path, compressed_data.as_bytes()).unwrap();

    println!(
        "Reduced size from {} to {} (ratio of {})",
        pixels565.as_bytes().len(),
        compressed_data.len(),
        (pixels565.as_bytes().len() as f32) / (compressed_data.len() as f32)
    );
}

fn process_animated_asset(_animated_asset_path: &Path) {}

pub fn rgb888torgb565(r8: u8, g8: u8, b8: u8) -> u16 {
    let r5 = (r8 >> 3) & 0b00011111;
    let g6 = (g8 >> 2) & 0b00111111;
    let b5 = (b8 >> 3) & 0b00011111;

    ((r5 as u16) << 11) | ((g6 as u16) << 5) | (b5 as u16)
}

#[cfg(test)]
mod tests {
    use image::EncodableLayout;

    #[test]
    fn check_decompression() {
        use image::ImageReader;
        let expected_vec = {
            let image = ImageReader::open("./input/espressif.png")
                .unwrap()
                .decode()
                .unwrap();
            let mut pixels565: Vec<u16> = Vec::new();
            for pixel in image.to_rgb8().pixels() {
                pixels565.push(super::rgb888torgb565(pixel[0], pixel[1], pixel[2]));
            }
            pixels565
        };
        let expected_bytes = expected_vec.as_bytes();

        let compressed_bytes = include_bytes!("../output/espressif.bin").as_bytes();
        let mut output_bytes = [0_u8; 128 * 128 * 2];
        let _bytes_wrote = miniz_oxide::inflate::decompress_slice_iter_to_slice(
            &mut output_bytes,
            core::iter::once(compressed_bytes),
            false,
            false,
        )
        .unwrap();

        assert_eq!(expected_bytes.len(), 128 * 128 * 2);

        assert_eq!(output_bytes, expected_bytes);
    }
}
