use crate::Compressor;
use image::{DynamicImage, EncodableLayout as _, ImageReader};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{self, DirEntry},
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

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    image_width: u16,
    image_height: u16,
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

// Private functions for static asset processing
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
        let file_name_lowercase = self.get_static_asset_name_lowercase(static_asset_path);

        match self.determine_image_file_format(&file_name_lowercase) {
            Some(_image_file_format) => (),
            None => {
                println!("Skipping non-image file {}", file_name_lowercase);
                return;
            }
        }
        println!("Compressing static asset {}", file_name_lowercase);

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

        if !is_snake_case(&file_name_no_ext) {
            panic!("Asset name must be snake_case: {}", file_name_lowercase);
        }

        let image = ImageReader::open(static_asset_path)
            .expect(format!("Failed to open image {}", file_name_lowercase).as_str())
            .decode()
            .expect(format!("Failed to decode image {}", file_name_lowercase).as_str());

        self.generate_static_asset_bin(output_dir, &file_name_lowercase, &image, compressor);

        self.generate_static_asset_json(output_dir, &file_name_lowercase, &image);
    }

    /// Get the file name of the provided path including the extension, which is
    /// converted to be lowercase. Panics if the file name cannot be obtained or
    /// cannot be converted to utf8
    fn get_static_asset_name_lowercase(&self, static_asset_path: &Path) -> String {
        static_asset_path
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
            .to_ascii_lowercase()
    }

    fn determine_image_file_format(&self, file_name_lowercase: &String) -> Option<ImageFileFormat> {
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
            Ok(file_format) => Some(file_format),
            Err(_e) => None,
        }
    }

    fn generate_static_asset_json(
        &self,
        output_dir: &Path,
        file_name_lowercase: &String,
        image: &DynamicImage,
    ) {
        let output_json_path =
            PathBuf::from(output_dir).join(Path::new(&file_name_lowercase).with_extension("json"));

        let json_data = JsonData {
            image_height: image.height() as u16,
            image_width: image.width() as u16,
        };
        let json_data_string = serde_json::to_string_pretty(&json_data).unwrap();
        let json_data_bytes = json_data_string.as_bytes();

        fs::write(output_json_path, json_data_bytes).unwrap();
    }

    fn generate_static_asset_bin<C: Compressor>(
        &mut self,
        output_dir: &Path,
        file_name_lowercase: &String,
        image: &DynamicImage,
        compressor: &C,
    ) where
        <C as Compressor>::Error: fmt::Debug,
    {
        let uncompressed_data = convert_image_to_bytes(&image, &self.target_color_format);

        let compressed_data = compressor.compress(&uncompressed_data.as_bytes()).unwrap();

        self.total_compressed_bytes += compressed_data.len() as u32;

        let output_bin_path =
            PathBuf::from(output_dir).join(Path::new(&file_name_lowercase).with_extension("bin"));

        fs::write(output_bin_path, compressed_data.as_bytes()).unwrap();
    }
}

// Private functions for animated asset processing
impl AssetProcessor {
    fn process_animated_asset<C: crate::Compressor>(
        &mut self,
        animated_asset_path: &Path,
        output_dir: &Path,
        compressor: &C,
    ) where
        <C as Compressor>::Error: fmt::Debug,
    {
        let animation_name_lowercase = self.get_animated_asset_name_lowercase(animated_asset_path);

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
        let mut animation_image_file_format: Option<ImageFileFormat> = None;

        // Frame numbers will be added to this vec in a non-deterministic order
        let mut processed_frame_numbers: Vec<u32> = Vec::new();

        // read_dir lists files in a non-deterministic (and likely non-alphabetical) order
        for frame in fs::read_dir(animated_asset_path).unwrap() {
            let frame = frame.unwrap();
            if !self.entry_is_file(&frame) {
                println!(
                    "Skipping non-file item `{}`",
                    frame.path().to_str().unwrap_or("unknown")
                );
                continue;
            };

            // Get file name as a string with no extension
            let file_name = self.get_frame_filename_lowercase_no_ext(&frame);

            let frame_image_file_format = match self.get_frame_image_file_format(&frame) {
                Some(image_format) => image_format,
                None => {
                    println!(
                        "Skipping non-image file `{}`",
                        frame.path().to_str().unwrap_or("unknown")
                    );
                    continue;
                }
            };

            let frame_number = self.get_frame_number(&file_name).expect(
                format!(
                    "Failed to get frame number for animation `{}`, frame file name `{}`",
                    animation_name_lowercase, file_name
                )
                .as_str(),
            );
            if processed_frame_numbers.contains(&frame_number) {
                panic!(
                    "Animation `{}` contains duplicate frame number: {}",
                    animation_name_lowercase, frame_number
                );
            }
            processed_frame_numbers.push(frame_number);

            println!(
                "Compressing animation `{}` frame {}",
                animation_name_lowercase, frame_number
            );

            self.check_inconsistent_frame_image_file_types(
                &mut animation_image_file_format,
                &frame_image_file_format,
                &animation_name_lowercase,
            );

            // You were here

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

        if processed_frame_numbers.is_empty() {
            println!(
                "Warning: Found animation with no frames: `{}`",
                animated_asset_path.to_str().unwrap_or("unknown")
            );
            return;
        }

        processed_frame_numbers.sort();

        if processed_frame_numbers[0] == 0 {
            panic!("Frames should be numbered starting with 1, not 0");
        }

        // Ensure one of each frame exists
        for (index, frame_number) in processed_frame_numbers.iter().enumerate() {
            if *frame_number != (index as u32) + 1 {
                panic!(
                    "Missing frame {} in animation, path: {}",
                    (index as u32) + 1,
                    animated_asset_path.to_str().unwrap()
                );
            }
        }
    }

    fn get_animated_asset_name_lowercase(&self, animated_asset_path: &Path) -> String {
        animated_asset_path
            .file_name()
            .expect(
                format!(
                    "Failed to get file name animation {}",
                    animated_asset_path.to_str().unwrap()
                )
                .as_str(),
            )
            .to_str()
            .unwrap()
            .to_ascii_lowercase()
    }

    fn entry_is_file(&self, entry: &DirEntry) -> bool {
        entry
            .file_type()
            .expect(
                format!(
                    "Failed to determine file type for frame `{}`",
                    entry.path().to_str().unwrap_or("unknown")
                )
                .as_str(),
            )
            .is_file()
    }

    fn get_frame_filename_lowercase_no_ext(&self, entry: &DirEntry) -> String {
        entry
            .path()
            .with_extension("")
            .file_name()
            .unwrap()
            .to_str()
            .expect(
                format!(
                    "Failed to convert frame file name to str for frame `{}`",
                    entry.path().to_str().unwrap_or("unknown")
                )
                .as_str(),
            )
            .to_lowercase()
    }

    fn get_frame_image_file_format(&self, entry: &DirEntry) -> Option<ImageFileFormat> {
        let file_format = match entry.path().extension() {
            Some(file_format_os_str) => file_format_os_str.to_str().unwrap().to_string(),
            None => String::new(),
        };

        file_format.parse::<ImageFileFormat>().ok()
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

    fn check_inconsistent_frame_image_file_types(
        &self,
        animation_image_file_format: &mut Option<ImageFileFormat>,
        frame_image_file_format: &ImageFileFormat,
        animation_name: &String,
    ) {
        match animation_image_file_format {
            Some(image_file_format) => {
                // Ensure this frame has the same image file format as previous frame(s)
                if image_file_format != frame_image_file_format {
                    panic!(
                        "Animation `{}` contains both {} and {} image file formats (all frames should have the same file format)",
                        animation_name,
                        image_file_format,
                        frame_image_file_format,
                    )
                };
            }
            None => {
                *animation_image_file_format = {
                    // This is the first animation frame found, so set the variable
                    Some(frame_image_file_format.clone())
                };
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
