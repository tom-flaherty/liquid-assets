use image::{EncodableLayout as _, ImageReader};
use std::{fs, path::Path};
use strum_macros::EnumString;

/// These are the image formats supported by the image crate
#[derive(EnumString, Debug, PartialEq, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum ImageFormat {
    Avif,
    Bmp,
    Dds,
    Exr,
    Ff,
    Gif,
    Hdr,
    Ico,
    Jpeg,
    Png,
    Pnm,
    Goi,
    Tga,
    Tiff,
    Webp,
}

pub enum TargetColorFormat {
    Rgb565,
}

pub struct AssetProcessor {
    target_color_format: TargetColorFormat,
    // Some stats
    static_assets_found: u32,
    animated_assets_found: u32,
    total_animation_frames: u32,
    total_uncompressed_bytes: u32,
    total_compressed_bytes: u32,
}

// Public functions
impl AssetProcessor {
    pub fn new(target_color_format: TargetColorFormat) -> Self {
        AssetProcessor {
            target_color_format,

            static_assets_found: 0,
            animated_assets_found: 0,
            total_animation_frames: 0,
            total_uncompressed_bytes: 0,
            total_compressed_bytes: 0,
        }
    }

    pub fn process(&mut self, input_path: &Path, output_dir: &Path) {
        for asset in fs::read_dir(input_path).unwrap() {
            let asset = asset.unwrap();
            let file_type = asset.file_type().unwrap();
            if file_type.is_file() {
                self.process_static_asset(Path::new(&asset.path()), output_dir);
                self.static_assets_found += 1;
            } else if file_type.is_dir() {
                self.process_animated_asset(Path::new(&asset.path()), output_dir);
                self.animated_assets_found += 1;
            } else {
                panic!()
            }
        }
    }

    pub fn print_stats(&self) {
        println!("Asset Processor:");
        println!("    {} static assets", self.static_assets_found);
        println!("    {} animated assets", self.animated_assets_found);
        println!("    {} animation frames", self.total_animation_frames);
        println!(
            "    {} total assets",
            self.static_assets_found + self.animated_assets_found
        );
        println!("    {} uncompressed bytes", self.total_uncompressed_bytes);
        println!("    {} compressed bytes", self.total_compressed_bytes);
        println!(
            "    {} avg. compression ratio",
            (self.total_uncompressed_bytes as f32) / (self.total_compressed_bytes as f32)
        );
    }
}

// Private functions
impl AssetProcessor {
    fn process_static_asset(&mut self, static_asset_path: &Path, output_dir: &Path) {
        let image = ImageReader::open(static_asset_path)
            .unwrap()
            .decode()
            .unwrap();
        let width = image.width();
        let height = image.height();

        let mut uncompressed_data: Vec<u8> = Vec::new();
        match self.target_color_format {
            TargetColorFormat::Rgb565 => {
                // For stats add uncompressed image size
                self.total_uncompressed_bytes += width * height * 2;

                // Convert image to rgb565
                let mut bytes_pushed = 0;
                for pixel in image.to_rgb8().pixels() {
                    for byte in rgb888_to_rgb565(pixel[0], pixel[1], pixel[2])
                        .to_be_bytes()
                        .into_iter()
                    {
                        uncompressed_data.push(byte);
                        bytes_pushed += 1;
                    }
                }
                assert_eq!(bytes_pushed, (width * height * 2) as usize);
            }
        };

        // Compress
        let compressed_data =
            miniz_oxide::deflate::compress_to_vec(uncompressed_data.as_bytes(), 10);

        self.total_compressed_bytes += compressed_data.len() as u32;

        let output_filename = static_asset_path.with_extension("bin");
        let output_filename = output_filename.file_name().unwrap();
        let mut output_path = output_dir.to_path_buf();
        output_path.push(output_filename);
        fs::write(output_path, compressed_data.as_bytes()).unwrap();
    }

    fn process_animated_asset(&mut self, animated_asset_path: &Path, output_dir: &Path) {
        // TODO create the directory for the output frames to go
        let mut animation_dir = output_dir.to_path_buf();
        animation_dir.push(animated_asset_path.file_name().unwrap());
        fs::create_dir(animation_dir.as_path()).unwrap();

        // Variable to ensure all images are the same type
        let mut animation_frame_file_format: Option<ImageFormat> = None;
        let mut frame_numbers: Vec<u32> = Vec::new();

        // Note that read_dir doesn't read files in any particular order (i.e. won't be alphabetical)
        for entry in fs::read_dir(animated_asset_path).unwrap() {
            let entry = entry.unwrap();

            let file_name = entry.file_name().to_str().unwrap().to_lowercase();
            let file_format = match entry.path().extension() {
                Some(file_format_os_str) => file_format_os_str.to_str().unwrap().to_string(),
                None => String::new(),
            };

            let entry_image_format = match file_format.parse::<ImageFormat>() {
                Ok(image_format) => image_format,
                Err(_e) => {
                    println!(
                        "Skipping file `{}` as it's not a valid image file",
                        file_name
                    );
                    continue;
                }
            };

            match animation_frame_file_format {
                Some(format) => {
                    if format != entry_image_format {
                        panic!(
                            "Animation contains more than one image type {}",
                            animated_asset_path.to_str().unwrap()
                        )
                    }
                }
                None => animation_frame_file_format = Some(entry_image_format),
            }

            let entry_file_name_no_ext = entry.path().with_extension("");
            let entry_file_name_no_ext = entry_file_name_no_ext
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let frame_number = {
                // Trim the numeric ending of the file name
                let trimmed_string = entry_file_name_no_ext.trim_end_matches(char::is_numeric);
                if trimmed_string.is_empty() {
                    panic!(
                        "Invalid animation frame name (expected non-numeric beginning) {:?}",
                        file_name
                    );
                }
                // Use this to get the numeric part
                let numeric_suffix = &entry_file_name_no_ext[trimmed_string.len()..];
                match numeric_suffix.parse::<u32>() {
                    Ok(frame_number) => frame_number,
                    Err(_e) => panic!(
                        "Invalid animation frame name (expected positive numeric ending) {:?}",
                        file_name
                    ),
                }
            };
            frame_numbers.push(frame_number);

            // TODO compress the frame
            let image = ImageReader::open(entry.path()).unwrap().decode().unwrap();
            let width = image.width();
            let height = image.height();

            let mut uncompressed_data: Vec<u8> = Vec::new();
            match self.target_color_format {
                TargetColorFormat::Rgb565 => {
                    // For stats add uncompressed image size
                    self.total_uncompressed_bytes += width * height * 2;

                    // Convert image to rgb565
                    let mut bytes_pushed = 0;
                    for pixel in image.to_rgb8().pixels() {
                        for byte in rgb888_to_rgb565(pixel[0], pixel[1], pixel[2])
                            .to_be_bytes()
                            .into_iter()
                        {
                            uncompressed_data.push(byte);
                            bytes_pushed += 1;
                        }
                    }
                    assert_eq!(bytes_pushed, (width * height * 2) as usize);
                }
            };

            let compressed_data =
                miniz_oxide::deflate::compress_to_vec(uncompressed_data.as_bytes(), 10);

            self.total_compressed_bytes += compressed_data.len() as u32;
            self.total_animation_frames += 1;

            // let output_filename = static_asset_path.with_extension("bin");
            let mut output_path = animation_dir.clone();
            output_path.push(format!("FRAME{}.bin", frame_number));
            fs::write(output_path, compressed_data.as_bytes()).unwrap();

            // TODO write the iterator code in the proc macro crate
            // TODO write the compressor and decompressor traits so users provide their own
            // TODO snake case check?
        }
        frame_numbers.sort();
        if frame_numbers[0] == 0 {
            panic!("Frames should be numbered starting with 1, not 0");
        }
        for (index, frame_number) in frame_numbers.iter().enumerate() {
            if *frame_number != (index as u32) + 1 {
                panic!(
                    "Missing frame {} in animation, path: {}",
                    (index as u32) + 1,
                    animated_asset_path.to_str().unwrap()
                );
            }
        }
    }
}

fn rgb888_to_rgb565(r8: u8, g8: u8, b8: u8) -> u16 {
    let r5 = (r8 >> 3) & 0b00011111;
    let g6 = (g8 >> 2) & 0b00111111;
    let b5 = (b8 >> 3) & 0b00011111;

    ((r5 as u16) << 11) | ((g6 as u16) << 5) | (b5 as u16)
}

// fn is_snake_case(str: &String) -> bool {
//     if str.is_empty() {
//         false
//     } else {
//         str.chars()
//             .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
//             && !str.starts_with(|c: char| c.is_ascii_digit())
//             && !str.contains("__")
//             && !str.ends_with('_')
//     }
// }
