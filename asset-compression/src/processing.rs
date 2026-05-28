use crate::Compressor;
use image::{DynamicImage, EncodableLayout as _, ImageReader};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use strum_macros::{Display, EnumString};

pub enum TargetColorFormat {
    Rgb565,
}

/// These are the image formats supported by the image crate
#[derive(EnumString, Debug, PartialEq, Clone, Copy, Display)]
#[strum(serialize_all = "snake_case")]
enum ImageFileFormat {
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

pub struct AssetProcessor {
    target_color_format: TargetColorFormat,
    // Statistics
    static_assets_found: u32,
    animated_assets_found: u32,
    total_animation_frames: u32,
    total_uncompressed_bytes: u32,
    total_compressed_bytes: u32,
    time_taken: Option<Duration>,
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
            time_taken: None,
        }
    }

    pub fn process<C: crate::Compressor>(
        &mut self,
        input_path: &Path,
        output_dir: &Path,
        compressor: &C,
    ) where
        <C as crate::Compressor>::Error: fmt::Debug,
    {
        let start_time = Instant::now();
        for asset in fs::read_dir(input_path).unwrap() {
            let asset = asset.unwrap();
            let file_type = asset.file_type().unwrap();
            if file_type.is_file() {
                self.process_static_asset(Path::new(&asset.path()), output_dir, compressor);
                self.static_assets_found += 1;
            } else if file_type.is_dir() {
                self.process_animated_asset(Path::new(&asset.path()), output_dir, compressor);
                self.animated_assets_found += 1;
            } else {
                println!(
                    "Skipping {} as it is neither a directory nor a file",
                    asset.path().to_str().unwrap()
                );
            }
        }
        self.time_taken = Some(start_time.elapsed());
    }

    pub fn generate_stats(&self) -> String {
        let total_assets = self.static_assets_found + self.animated_assets_found;
        let compression_ratio =
            (self.total_uncompressed_bytes as f32) / (self.total_compressed_bytes as f32);
        format!(
            "Compression Statistics:
{} static assets
{} animated assets
{} animation frames
{} total assets
{} uncompressed bytes
{} compressed bytes
{} avg. compression ratio
Took {:?}",
            self.static_assets_found,
            self.animated_assets_found,
            self.total_animation_frames,
            total_assets,
            self.total_uncompressed_bytes,
            self.total_compressed_bytes,
            compression_ratio,
            self.time_taken.unwrap_or_else(|| Duration::MAX)
        )
    }
}

// Private functions
impl AssetProcessor {
    fn process_static_asset<C: Compressor>(
        &mut self,
        static_asset_path: &Path,
        output_dir: &Path,
        compressor: &C,
    ) where
        <C as Compressor>::Error: fmt::Debug,
    {
        // Ensure the file extension is a valid image file
        let file_name_lowercase = static_asset_path
            .file_name()
            .expect(
                format!(
                    "Failed to get file name for {}",
                    static_asset_path.to_str().unwrap()
                )
                .as_str(),
            )
            .to_str()
            .expect(
                format!(
                    "Failed to convert file name to str {}",
                    static_asset_path.to_str().unwrap()
                )
                .as_str(),
            )
            .to_ascii_lowercase();

        let file_format = Path::new(&file_name_lowercase)
            .extension()
            .expect(format!("Could not get extension for {}", file_name_lowercase).as_str())
            .to_str()
            .expect(
                format!(
                    "Failed to convert extension to str for {}",
                    file_name_lowercase
                )
                .as_str(),
            );

        match file_format.parse::<ImageFileFormat>() {
            Ok(_file_format) => println!("Compressing {}", file_name_lowercase),
            Err(_e) => {
                println!("Skipping non-image file {}", file_name_lowercase);
                return;
            }
        }

        // Ensure the asset name is snake case
        let file_name_no_ext = Path::new(&file_name_lowercase)
            .with_extension("")
            .file_name()
            .expect(format!("Failed to get file name for {}", file_name_lowercase).as_str())
            .to_str()
            .expect(format!("Failed to convert file name to str {}", file_name_lowercase).as_str())
            .to_string();

        if !is_snake_case(&file_name_no_ext) {
            panic!("Asset name must be snake_case: {}", file_name_lowercase);
        }

        let image = ImageReader::open(static_asset_path)
            .expect(format!("Failed to open image {}", file_name_lowercase).as_str())
            .decode()
            .expect(format!("Failed to decode image {}", file_name_lowercase).as_str());

        let uncompressed_data = convert_image_to_bytes(&image, &self.target_color_format);

        let compressed_data = compressor.compress(&uncompressed_data.as_bytes()).unwrap();

        self.total_compressed_bytes += compressed_data.len() as u32;

        let output_path =
            PathBuf::from(output_dir).join(Path::new(&file_name_lowercase).with_extension("bin"));

        fs::write(output_path, compressed_data.as_bytes()).unwrap();
    }

    fn process_animated_asset<C: crate::Compressor>(
        &mut self,
        animated_asset_path: &Path,
        output_dir: &Path,
        compressor: &C,
    ) where
        <C as Compressor>::Error: fmt::Debug,
    {
        let animation_name_lowercase = animated_asset_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();

        if !is_snake_case(&animation_name_lowercase) {
            panic!(
                "Asset name must be snake_case: {}",
                animation_name_lowercase
            );
        }

        let animated_output_dir = output_dir
            .to_path_buf()
            .join(animated_asset_path.file_name().unwrap());

        fs::create_dir(animated_output_dir.as_path()).unwrap();

        // Used to ensure all images have the same file format
        let mut image_file_format: Option<ImageFileFormat> = None;

        // Frame numbers will be added to this vec in a non-deterministic order
        let mut frame_numbers: Vec<u32> = Vec::new();

        // read_dir lists files in a non-deterministic (and likely non-alphabetical) order
        for frame in fs::read_dir(animated_asset_path).unwrap() {
            let frame = frame.unwrap();

            if !frame
                .file_type()
                .expect(
                    format!(
                        "Failed to determine file type for frame in {}",
                        animation_name_lowercase
                    )
                    .as_str(),
                )
                .is_file()
            {
                panic!(
                    "Unexpected non-file item in animation directory {}",
                    animation_name_lowercase
                )
            }

            // Get file name as a string with no extension
            let file_name = frame
                .path()
                .with_extension("")
                .file_name()
                .unwrap()
                .to_str()
                .expect(
                    format!(
                        "Failed to convert frame file name to str for animation {}",
                        animation_name_lowercase
                    )
                    .as_str(),
                )
                .to_lowercase();

            let file_format = match frame.path().extension() {
                Some(file_format_os_str) => file_format_os_str.to_str().unwrap().to_string(),
                None => String::new(),
            };

            let frame_image_format = match file_format.parse::<ImageFileFormat>() {
                Ok(image_format) => image_format,
                Err(_e) => {
                    println!(
                        "Skipping file `{}` as it's not a valid image file",
                        frame.file_name().to_str().unwrap()
                    );
                    continue;
                }
            };

            let frame_number = self.get_frame_number(&file_name).expect(
                format!(
                    "Failed to get frame number for animation {} file name {}",
                    animation_name_lowercase, file_name
                )
                .as_str(),
            );
            if frame_numbers.contains(&frame_number) {
                panic!(
                    "Animation {} contains duplicate frame number {}",
                    animation_name_lowercase, frame_number
                );
            }
            frame_numbers.push(frame_number);

            println!(
                "Compressing animation `{}` frame {}",
                animation_name_lowercase, frame_number
            );

            match image_file_format {
                Some(format) => {
                    // Ensure this frame has the same image file format as previous frame(s)
                    if format != frame_image_format {
                        panic!(
                            "Animation `{}` contains both {} and {} image file formats (all frames should have the same file format)",
                            animation_name_lowercase, frame_image_format, format,
                        )
                    }
                }
                None => {
                    image_file_format = {
                        // This is the first animation frame found, so set the variable
                        Some(frame_image_format)
                    }
                }
            }

            let image = ImageReader::open(frame.path())
                .expect(format!("Failed to open image {}", animation_name_lowercase).as_str())
                .decode()
                .expect(format!("Failed to decode image {}", animation_name_lowercase).as_str());

            let uncompressed_data = convert_image_to_bytes(&image, &self.target_color_format);

            let compressed_data = compressor.compress(&uncompressed_data.as_bytes()).unwrap();

            // Statistics
            self.total_uncompressed_bytes += uncompressed_data.len() as u32;
            self.total_compressed_bytes += compressed_data.len() as u32;
            self.total_animation_frames += 1;

            let frame_output_path =
                animated_output_dir.join(Path::new(&format!("FRAME{}.bin", frame_number)));

            fs::write(frame_output_path, compressed_data.as_bytes()).unwrap();
        }

        frame_numbers.sort();

        if frame_numbers[0] == 0 {
            panic!("Frames should be numbered starting with 1, not 0");
        }

        // Ensure one of each frame exists
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

    fn get_frame_number(&self, frame_file_name_str: &String) -> Result<u32, ()> {
        let trimmed_string = frame_file_name_str.trim_end_matches(char::is_numeric);
        if trimmed_string.is_empty() {
            Err(())
        } else {
            let numeric_suffix = &frame_file_name_str[trimmed_string.len()..];
            match numeric_suffix.parse::<u32>() {
                Ok(frame_number) => Ok(frame_number),
                Err(_e) => Err(()),
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

fn convert_image_to_bytes(
    image: &DynamicImage,
    target_color_format: &TargetColorFormat,
) -> Vec<u8> {
    let mut image_data: Vec<u8> = Vec::new();
    match target_color_format {
        TargetColorFormat::Rgb565 => {
            // Convert image to rgb565
            let mut bytes_pushed = 0;
            for pixel in image.to_rgb8().pixels() {
                for byte in rgb888_to_rgb565(pixel[0], pixel[1], pixel[2])
                    .to_be_bytes()
                    .into_iter()
                {
                    image_data.push(byte);
                    bytes_pushed += 1;
                }
            }
            assert_eq!(bytes_pushed, (image.width() * image.height() * 2) as usize);
        }
    };
    image_data
}

fn is_snake_case(str: &String) -> bool {
    if str.is_empty() {
        false
    } else {
        str.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && !str.starts_with(|c: char| c.is_ascii_digit())
            && !str.contains("__")
            && !str.ends_with('_')
    }
}
