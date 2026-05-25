use std::{fs, path::Path};

use image::{EncodableLayout as _, ImageReader};

use crate::rgb888torgb565;

pub fn process_assets(input_path: &Path, output_dir: &Path) {
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