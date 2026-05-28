Rust compression libraries can be found here:

https://crates.io/categories/compression

You can convert a gif to frames using:

`ffmpeg -i mygif.gif frame_%04d.png`

Or

`ffmpeg -i mygif.gif -start_number 1 -vf scale=128:128 frame_%04d.png`

# TODO

- You've got a mix of 'graphics' and 'assets'. Change everything to assets

- Add postcard serialization so you can embed image width and height into the compressed data?
- Add image width and height for all assets

- Ensure all frames in an animation are the same size

- Add MIPIDSI library to the c3 example

- Flood docstrings everywhere, including the proc macro generated code

- Add careful support for turning decompressed data into a embedded_graphics::Image.
Maybe this should be a feature?

- Give the repo a witty name

# Long Term TODO

- Support for displays other using colour formats other than RGB565
- Support for transparency
- Add a way to build assets without adding to build.rs
- Support for bitmaps?
