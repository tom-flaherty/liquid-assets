use quote::quote;
use std::{
    fs::{self, DirEntry},
    os::unix::process,
    path::{Path, PathBuf},
};
use syn::{Error, LitStr, parse::Parse};

struct MacroArgs {
    relative_path: String,
}

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str: LitStr = input.parse()?;
        Ok(MacroArgs {
            relative_path: lit_str.value(),
        })
    }
}

#[proc_macro]
pub fn include_graphics(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: MacroArgs = syn::parse_macro_input!(input as MacroArgs);

    let cargo_manifest_str = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_manifest_path = Path::new(&cargo_manifest_str);

    let path_extension = Path::new(&input.relative_path);

    let mut graphics_dir = cargo_manifest_path.to_path_buf();
    graphics_dir.push(path_extension);
    let graphics_dir = Path::new(&graphics_dir);

    let mut struct_quotes: Vec<proc_macro2::TokenStream> = Vec::new();
    let read_dir = match fs::read_dir(graphics_dir) {
        Ok(read_dir) => read_dir,
        Err(e) => {
            return Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "Failed to read directory {:?}: {:?}",
                    graphics_dir.to_str(),
                    e
                ),
            )
            .to_compile_error()
            .into();
        }
    };

    for item in read_dir {
        let item = item.unwrap();
        let file_type = item.file_type().unwrap();
        if file_type.is_dir() {
            todo!()
        } else if file_type.is_file() {
            struct_quotes.push(process_static_asset(item));
        }
    }

    quote! {
        pub mod assets {
            pub struct StaticAsset {
                pub bytes: &'static [u8],
            }

            #(#struct_quotes)*
        }
    }
    .into()
}

fn process_static_asset(asset: DirEntry) -> proc_macro2::TokenStream {
    let asset_name_str = String::from(asset.path().file_stem().unwrap().to_str().unwrap());

    if !is_snake_case(&asset_name_str) {
        return Error::new(
            proc_macro2::Span::call_site(),
            format!("Asset name is not valid snake_case: {:?}", asset_name_str),
        )
        .to_compile_error()
        .into();
    }

    let asset_name_token_stream: proc_macro2::TokenStream = asset_name_str.parse().unwrap();

    // let binary_path: proc_macro2::TokenStream =
    let binary_path: String =
        asset.path().to_str().unwrap().to_owned();//.parse().unwrap();

    eprintln!("{:?}", binary_path);

    quote! {
        pub const #asset_name_token_stream: StaticAsset = StaticAsset {
            bytes: include_bytes!(#binary_path).as_slice(),
        };
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
