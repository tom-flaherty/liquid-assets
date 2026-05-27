use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};
use syn::{Expr, LitInt, LitStr, Token, parse::Parse};

enum BufferSizeParam {
    Expr(Expr),
    LitInt(usize),
}

struct MacroArgs {
    graphics_path: String,
    buffer_size: BufferSizeParam,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let buffer_size = match input.parse::<LitInt>() {
            Ok(lit_int) => BufferSizeParam::LitInt(lit_int.base10_parse()?),
            Err(_e) => BufferSizeParam::Expr(input.parse::<Expr>()?),
        };

        Ok(MacroArgs {
            graphics_path: lit_str.value(),
            buffer_size,
        })
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn include_graphics(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let MacroArgs {
        graphics_path,
        buffer_size,
    } = syn::parse_macro_input!(input as MacroArgs);

    let mut target_dir: PathBuf = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(path_str) => path_str,
        Err(e) => abort_call_site!(
            "Could not read environment variable CARGO_MANIFEST_DIR: {:?}",
            e
        ),
    }
    .into();
    target_dir.push(Path::new(&graphics_path));

    let target_dir_read = match fs::read_dir(&target_dir) {
        Ok(read_dir) => read_dir,
        Err(e) => {
            abort_call_site!("Failed to read directory {:?}: {:?}", target_dir, e)
        }
    };

    let mut struct_quotes: Vec<proc_macro2::TokenStream> = Vec::new();
    for item in target_dir_read {
        let item = item.unwrap();
        let file_type = item.file_type().unwrap();
        if file_type.is_dir() {
            struct_quotes.push(process_animated_asset(item, &buffer_size));
        } else if file_type.is_file() {
            struct_quotes.push(process_static_asset(item));
        }
    }

    let struct_definitions = define_structs();

    quote! {
        pub mod assets {
            use asset_decompression::Decompressor;
            #struct_definitions
            #(#struct_quotes)*
        }
    }
    .into()
}

fn process_static_asset(asset: DirEntry) -> proc_macro2::TokenStream {
    let asset_name = process_asset_name(&asset);

    let path_to_bin: String = match asset.path().to_str() {
        Some(path_as_str) => path_as_str,
        None => abort_call_site!("Unable to convert path to string: {:?}", asset.path()),
    }
    .to_owned();

    quote! {
        pub const #asset_name: StaticAsset = StaticAsset {
            data: include_bytes!(#path_to_bin).as_slice(),
        };
    }
}

fn process_animated_asset(
    asset: DirEntry,
    buffer_size: &BufferSizeParam,
) -> proc_macro2::TokenStream {
    let asset_name = process_asset_name(&asset);

    let total_frames = match fs::read_dir(asset.path()) {
        Ok(read_dir) => read_dir,
        Err(e) => abort_call_site!(
            "Could not read animated asset directory {:?}: {:?}",
            asset.path(),
            e
        ),
    }
    .count();

    // Data for each frame is added with include bytes
    let mut frame_data: Vec<proc_macro2::TokenStream> = Vec::new();

    for frame_number in 1..(total_frames + 1) {
        let frame_dir = asset.path().clone();

        let frame_bin_path = frame_dir.join(Path::new(&format!("FRAME{}.bin", frame_number)));

        let frame_bin_path_str = match frame_bin_path.to_str() {
            Some(frame_path_str) => frame_path_str,
            None => abort_call_site!("Failed to convert frame path to str"),
        };

        // Surround the path with quote marks
        let frame_bin_path_str = format!("\"{}\"", frame_bin_path_str);

        let frame_path = match frame_bin_path_str.parse::<proc_macro2::TokenStream>() {
            Ok(frame_path_token_stream) => frame_path_token_stream,
            Err(e) => abort_call_site!(
                "Failed to convert frame path str into token stream: {:?}",
                e
            ),
        };

        frame_data.push(quote! {
            include_bytes!(#frame_path).as_slice(),
        });
    }

    // The AnimatedAsset struct requires a size at compile time. The user may have entered
    // a integer literal (which is used directly) or an expression which is accessed with super::
    let buffer_size = match buffer_size {
        BufferSizeParam::Expr(expr) => quote! {{ super::#expr }},
        BufferSizeParam::LitInt(lit_int) => quote! {#lit_int},
    };

    quote! {
        pub const #asset_name: AnimatedAsset<#buffer_size> = AnimatedAsset {
            frames: &[
                #(#frame_data)*
            ],
        };
    }
}

fn process_asset_name(asset_dir_entry: &DirEntry) -> proc_macro2::TokenStream {
    let asset_path = asset_dir_entry.path();
    let asset_file_stem = match asset_path.file_stem() {
        Some(file_stem) => file_stem,
        None => abort_call_site!(
            "Unable to get file stem for asset {:?}",
            asset_dir_entry.path()
        ),
    };
    let asset_name = match asset_file_stem.to_str() {
        Some(asset_name) => asset_name,
        None => abort_call_site!(
            "Unable to convert file stem into str {:?}",
            asset_dir_entry.path()
        ),
    }
    .to_string();

    if !is_snake_case(&asset_name) {
        abort_call_site!("Asset name is not valid snake_case: {:?}", asset_name);
    }

    let asset_name = asset_name.to_ascii_uppercase();

    asset_name.parse().unwrap()
}

fn define_structs() -> proc_macro2::TokenStream {
    quote! {
        pub struct StaticAsset {
            data: &'static [u8],
        }
        impl StaticAsset {
            pub fn get_comressed_data(&self) -> &'static [u8] {
                self.data
            }
            pub fn decompress<const N: usize>(
                &self,
                buffer: &mut [u8; N],
                decompressor: &impl Decompressor,
            ) -> Result<usize, ()> {
                decompressor.decompress(buffer, self.data)
            }
        }

        pub struct AnimatedAsset<const N: usize> {
            frames: &'static [&'static [u8]],
        }
        impl<const N: usize> AnimatedAsset<N> {
            pub const fn get_number_of_frames(&self) -> usize {
                self.frames.len()
            }
            pub fn decompress_frame(
                &self,
                frame_number: usize,
                buffer: &mut[u8; N],
                decompressor: &impl Decompressor
            ) -> Result<usize, ()> {
                if frame_number < self.frames.len() {
                    decompressor.decompress(buffer, self.frames[frame_number])
                } else {
                    Err(())
                }
            }
            pub fn get_compressed_frame_data(
                &self,
                frame_number: usize,
            ) -> Option<&'static [u8]> {
                if frame_number < self.frames.len() {
                    Some(self.frames[frame_number])
                } else {
                    None
                }
            }
            #[doc = "This is a test docstring"]
            pub fn copy_compressed_frame_data_to_buffer(
                &self,
                frame_number: usize,
                buffer: &mut[u8; N],
            ) -> Option<usize> {
                if frame_number < self.frames.len() {
                    let source_bytes = self.frames[frame_number as usize];
                    buffer[..source_bytes.len()].copy_from_slice(source_bytes);
                    Some(source_bytes.len())
                } else {
                    None
                }
            }
            pub fn as_iter(&self) -> FrameIterator {
                FrameIterator::new(self.frames)
            }
        }

        pub struct FrameIterator {
            frames: &'static [&'static [u8]],
            current_frame: usize,
        }
        impl FrameIterator {
            pub fn new(frames: &'static [&'static [u8]]) -> Self {
                Self {
                    frames,
                    current_frame: 0,
                }
            }
        }
        impl Iterator for FrameIterator {
            type Item = &'static [u8];

            fn next(&mut self) -> Option<Self::Item> {
                if self.current_frame < self.frames.len() {
                    let return_val = Some(self.frames[self.current_frame]);
                    self.current_frame += 1;
                    return_val
                } else {
                    None
                }
            }
        }
    }
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
