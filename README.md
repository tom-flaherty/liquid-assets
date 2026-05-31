# `liquid-assets`

`liquid-assets` is an assets pipeline for embedded Rust. It has two parts:

- `liquid-assets-deflate` is used to compress source images into binaries.
- `liquid-assets-inflate` provides a macro which loads these images and provides easy methods for decompressing them.

## What problem does it solve?

In my experience, to display animation frames on embedded Rust hardware I write a python script which converts each frame to a .bin file, and then I manually include each frame with the `include_bytes!` macro.

```rust
const ANIMATION_DATA = [
    include_bytes!("path/frame1.bin"),
    include_bytes!("path/frame2.bin"),
    ... // Rinse and repeat many times
];
```

This isn't great because if you need to change the animations you have to rewrite this. Not to mention that there may be hundreds of frames in an animation.

`liquid_assets` provides a pipeline which automates the compression and decompression of these assets.

`liquid_assets_deflate` provides the `build_assets` function which can be placed in `build.rs`, and will compress assets into .bin files. The user has to implement the `Compressor` trait, to implement a compression library. Some example implementations of this trait are included in /example/build.rs.

## Quick Example

```rust
// build.rs
struct MinizOxideCompressor {}
impl Compressor for MinizOxideCompressor {
    // The compression is infallible
    type Error = ();

    fn compress(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        const COMPRESSION_LEVEL: u8 = 5;
        Ok(miniz_oxide::deflate::compress_to_vec(
            input_bytes,
            COMPRESSION_LEVEL,
        ))
    }
}
fn main() {
    let mut compressor = MinizOxideCompressor {};
    build_assets(
        "./assets", // Source directory for assets
        "./asset-binaries", // Target directory for binaries
        TargetColorFormat::Rgb565, // Images will be converted to this colour format
        &mut compressor, // Pass a reference to the compressor
    );
    ... // Rest of build.rs
```

This will rebuild the assets whenever the assets source directory changes, or if the user runs `REBUILD_ASSETS=1 cargo build`.

Next, to load these animations in the embedded Rust code, simply call the `include_assets!` macro which is provided by `liquid_assets_inflate`.

```rust
use liquid_assets_inflate::include_assets;
const BUFFER_SIZE: usize = 135 * 135 * 2;
include_assets!("asset-binaries", BUFFER_SIZE);

fn main() {
    ... // Embedded setup here

    // Decompress a static asset
    let DecompressedData {
        bytes_wrote,
        width,
        height,
    } = assets::ESPRESSIF
        .decompress(&mut buffer, &decompressor)
        .unwrap();
    // You can now access the image as a slice of the buffer
    let data = buffer[..bytes_wrote];
    // It's up to the user to convert this into something that the display driver can use

    // You can also decompress animations as an iterator
    for (frame_index, frame) in assets::GITHUB.as_iter().enumerate() {
        let DecompressedData {
            bytes_wrote,
            width,
            height,
        } = frame.decompress(&mut buffer, &decompressor).unwrap();
        // Then display the frame
        // Then add a delay to maintain a steady framerate
    }
    ...
}
```

Note that in the Cargo.toml `liquid-assets-inflate` is added to `[dependencies]` and `liquid-assets-deflate` is added to `[build-dependencies]`.

```
[dependencies]
liquid-assets-inflate = { git = "git@github.com:tom-flaherty/liquid-assets.git", version = "0.1.1" }
[build-dependencies]
liquid-assets-deflate = { git = "git@github.com:tom-flaherty/liquid-assets.git", version = "0.1.1" }
```

## Assets Directory Format

In the following example, `espressif` is a static asset, whereas `github` and `loading` are animations. Images must already be the desired size. Frames must be named `snake_case` with a numeric suffix starting with 1.

```
assets
├── espressif.png
├── github
│   ├── frame_0001.png
│   ├── frame_0002.png
│   └── ...
└── loading
    ├── frame_0001.png
    ├── frame_0002.png
    └── ...
```

To use `liquid-assets`, first add `liquid-assets-deflate` to the `[dev-dependencies]` of your Cargo.toml.

# Long Term TODO

- The compression code could be refactored to use more structs, which may improve readablility
- Remove "... as u16" from compression code
- Support for displays other using colour formats other than RGB565
- Support for transparency
- Add a way to build assets without adding to build.rs
- Support for bitmaps?
- Prevent unwanted rebuilds
- Only rebuild the specific assets that changed (this would add a lot of complexity)

## Licence

This software is provided under the MIT Licence (see LICENCE file). If you find this project helpful, please give the repo a star.

## Contributing

Please raise an issue on github to discuss changes.

## Other notes

Only tested on Linux.

You can convert a gif to frames using:

`ffmpeg -i mygif.gif frame_%04d.png`

Or to also resize:

`ffmpeg -i mygif.gif -vf scale=128:128 frame_%04d.png`
