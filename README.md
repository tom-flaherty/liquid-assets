# `liquid-assets`

`liquid-assets` is an assets pipeline for embedded Rust. It has two parts:

- `liquid-assets-deflate` is used to compress source images into binaries. It uses `std` as it doesn't run on the embedded hardware.
- `liquid-assets-inflate` is used to decompress those binaries into images. It uses `no_std`.

Any `no_std` compatible compression library can be used.

## Example

This is a typical embedded Rust project structure:

```
.
├── assets
├── build.rs
├── Cargo.toml
├── rust-toolchain.toml
└── src
    ├── bin
    │   └── main.rs
    └── lib.rs
```

In the assets directory, there is an image called `espressif.png` (referred to as a static asset), and two animations - one called `github` and one called `loading` (referred to as animated assets). Each animation folder contains the frames to display that animation, with a number suffixed to the name.

```
assets
├── espressif.png
├── github
│   ├── credit.txt
│   ├── frame_0001.png
│   ├── frame_0002.png
│   └── ...
└── loading
    ├── credit.txt
    ├── frame_0001.png
    ├── frame_0002.png
    └── ...
```

To use `liquid-assets`, first add `liquid-assets-deflate` to the `[dev-dependencies]` of your Cargo.toml.

```
[build-dependencies]
liquid-assets-deflate = { git = "git@github.com:tom-flaherty/liquid-assets.git", version = "0.1.0" }
```

Next, in your `build.rs` file, implement the `liquid-assets_deflate::Compressor` trait onto a struct. You can probably just copy from `examples/build.rs` as examples for common compression libraries are included.

Run the `liquid_assets_deflate::include_assets_if_changed` function, providing these parameters:

- The assets source directory (relative to CARGO_MANIFEST_DIR)
- The assets binary directory (relative to CARGO_MANIFEST_DIR), which will be created. You may want to add this to your .gitignore
- The target colour format (currently only RGB565 is supported)
- A reference to a struct which has the `Compressor` trait implemented

```rust
use liquid_assets_deflate::{Compressor, TargetColorFormat, rebuild_assets_if_changed};

struct ZlibCompressor {}
impl Compressor for ZlibCompressor {
    // The compression is infallible
    type Error = ();

    fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        const COMPRESSION_LEVEL: u8 = 5;
        Ok(miniz_oxide::deflate::compress_to_vec(
            input_bytes,
            COMPRESSION_LEVEL,
        ))
    }
}

fn main() {
    let zlib_compressor = ZlibCompressor {};

    rebuild_assets_if_changed(
        "./assets",
        "./asset-binaries",
        TargetColorFormat::Rgb565,
        &zlib_compressor,
    );
```

Now when you run `cargo build`, the assets binaries will be built. The assets will only be rebuilt if:

- There is a change to the assets source directory (e.g. you add or remove an asset).
- You run `REBUILD_ASSETS=1 cargo build`
- Another part of the `build.rs` file triggers a rebuild (preventing this is WIP!)

Next, the deflate component, which uses proc-macro magic.

Add `liquid-assets-inflate` to your Cargo.toml:

```
[dependencies]
liquid-assets-inflate = { git = "git@github.com:tom-flaherty/liquid-assets.git", version = "0.1.0" }
```

In a source file (not inside of a function), invoke the `include_assets` macro, providing a struct which implements the `liquid_assets_inflate::Decompressor` trait. Again, there are some examples you can copy.

```rust
const BUFFER_SIZE: usize = 128 * 128 * 2;

struct ZlibDecompressor {}
impl Decompressor for ZlibDecompressor {
    type Error = miniz_oxide::inflate::TINFLStatus;

    fn decompress<const N: usize>(
        &self,
        buffer: &mut [u8; N],
        compressed_data: &[u8],
    ) -> Result<usize, Self::Error> {
        miniz_oxide::inflate::decompress_slice_iter_to_slice(
            buffer,
            core::iter::once(compressed_data),
            false,
            false,
        )
    }
}

liquid_assets_inflate::include_assets!("asset-binaries", BUFFER_SIZE);

pub fn run() {
    let mut buffer = [0_u8; BUFFER_SIZE];
    let decompressor = ZlibDecompressor {};
    // Decompress the `espressif` static asset into the buffer
    assets::ESPRESSIF
        .decompress(&mut buffer, &decompressor)
        .unwrap();
    // Decompress frame 3 of the `loading` animation into the buffer
    let bytes_written = assets::LOADING
        .decompress_frame(3, &mut buffer, &decompressor)
        .unwrap();
    // Loop through an animation by using a iterator
    for frame in assets::GITHUB.as_iter() {
        frame.decompress(&mut buffer, &decompressor).unwrap();
        // You could add a delay here to maintain a framerate
    }
}
```

TODO: Document the stuff created by the proc macro.

## BYOCompression Library

Rust compression libraries can be found here:

https://crates.io/categories/compression

Libraries should be `no_std` and `no_alloc`.

# TODO

- Add postcard serialization so you can embed image width and height into the compressed data?
- Add image width and height for all assets
- Ensure all frames in an animation are the same size
- Add MIPIDSI library to the c3 example
- Flood docstrings everywhere, including the proc macro generated code
- Add careful support for turning decompressed data into a embedded_graphics::Image.
Maybe this should be a feature?
- Give the repo a witty name
- Add a licence like a true professional
- Tart up the README

# Long Term TODO

- Support for displays other using colour formats other than RGB565
- Support for transparency
- Add a way to build assets without adding to build.rs
- Support for bitmaps?
- Prevent unwanted rebuilds

# Other notes

You can convert a gif to frames using:

`ffmpeg -i mygif.gif frame_%04d.png`

Or

`ffmpeg -i mygif.gif -start_number 1 -vf scale=128:128 frame_%04d.png`
