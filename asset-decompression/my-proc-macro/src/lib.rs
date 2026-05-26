use quote::quote;
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};
use syn::{Error, Expr, LitInt, LitStr, Token, parse::Parse};

enum BufferSizeParam {
    Expr(Expr),
    LitInt(usize),
}

struct MacroArgs {
    graphics_path: String,
    buffer_size: BufferSizeParam,
    // decompressor: proc_macro2::TokenStream,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let buffer_size = match input.parse::<LitInt>() {
            Ok(lit_int) => BufferSizeParam::LitInt(lit_int.base10_parse()?),
            Err(_e) => BufferSizeParam::Expr(input.parse::<Expr>()?),
        };
        // TODO adding the decompressor trait
        // input.parse::<Token![,]>()?;
        // let decompressor: proc_macro2::TokenStream = input.parse()?;

        Ok(MacroArgs {
            graphics_path: lit_str.value(),
            buffer_size,
            // decompressor,
        })
    }
}

#[proc_macro]
pub fn include_graphics(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let MacroArgs {
        graphics_path,
        buffer_size,
    } = syn::parse_macro_input!(input as MacroArgs);

    let mut target_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    target_dir.push(Path::new(&graphics_path));

    let mut struct_quotes: Vec<proc_macro2::TokenStream> = Vec::new();
    let read_dir = match fs::read_dir(&target_dir) {
        Ok(read_dir) => read_dir,
        Err(e) => {
            return Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to read directory {:?}: {:?}", target_dir, e),
            )
            .to_compile_error()
            .into();
        }
    };

    for item in read_dir {
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
            #struct_definitions
            #(#struct_quotes)*
        }
    }
    .into()
}

fn process_static_asset(asset: DirEntry) -> proc_macro2::TokenStream {
    let asset_name_str: String = asset
        .path()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    if !is_snake_case(&asset_name_str) {
        return Error::new(
            proc_macro2::Span::call_site(),
            format!("Asset name is not valid snake_case: {:?}", asset_name_str),
        )
        .to_compile_error()
        .into();
    }

    let asset_name_token_stream: proc_macro2::TokenStream =
        asset_name_str.to_ascii_uppercase().parse().unwrap();

    let binary_path: String = asset.path().to_str().unwrap().to_owned();

    quote! {
        pub const #asset_name_token_stream: StaticAsset = StaticAsset {
            bytes: include_bytes!(#binary_path).as_slice(),
        };
    }
}

fn process_animated_asset(
    asset: DirEntry,
    buffer_size: &BufferSizeParam,
) -> proc_macro2::TokenStream {
    let frame_count = fs::read_dir(asset.path()).unwrap().count();
    let animation_name_str: String = asset
        .path()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .to_ascii_uppercase();
    let animation_name_token_stream: proc_macro2::TokenStream = animation_name_str.parse().unwrap();
    let mut include_bytes_quotes: Vec<proc_macro2::TokenStream> = Vec::new();
    for frame_number in 1..(frame_count + 1) {
        let mut frame_path = asset.path().clone();
        frame_path.push(format!("FRAME{}.bin", frame_number));
        let frame_path_token_stream: proc_macro2::TokenStream =
            format!("\"{}\"", frame_path.to_str().unwrap())
                .parse()
                .unwrap();
        include_bytes_quotes.push(quote! {
            include_bytes!(#frame_path_token_stream).as_slice(),
        });
    }
    let animated_asset_generic = match buffer_size {
        BufferSizeParam::Expr(expr) => quote! {{ super::#expr }},
        BufferSizeParam::LitInt(lit_int) => quote! {#lit_int},
    };

    quote! {
        pub const #animation_name_token_stream: AnimatedAsset<#animated_asset_generic> = AnimatedAsset {
            frames: &[
                #(#include_bytes_quotes)*
            ],
        };
    }
}

fn define_structs() -> proc_macro2::TokenStream {
    quote! {
        pub struct StaticAsset {
            pub bytes: &'static [u8],
        }
        pub struct AnimatedAsset<const N: usize> {
            frames: &'static [&'static [u8]]
        }
        impl<const N: usize> AnimatedAsset<N> {
            pub const fn get_number_of_frames(&self) -> usize {
                self.frames.len()
            }
            pub fn get_frame(&self, frame_number: u32, buffer: &mut [u8; N]) -> Result<usize, ()> {
                if frame_number as usize >= self.frames.len() {
                    return Err(());
                } else {
                    let source_bytes = self.frames[frame_number as usize];
                    buffer[..source_bytes.len()].copy_from_slice(source_bytes);
                    Ok(source_bytes.len())
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
