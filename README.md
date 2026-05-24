Rust compression libraries can be found here:

https://crates.io/categories/compression

You can convert a gif to frames using:

`ffmpeg -i mygif.gif frame_%04d.png`

# TODO

- Generate the animation assets
- Better cargo tests
- macro which take graphics output file as a paramter, and generates module
with all the graphics. Animations are provided as an iterator
- Crate which provides build tools, which can be added to build.rs and automatically
rebuild assets when the input folder changes
- Should the crate which provides compression also provide the decompression method?
- How to support other display sizes, other RGB standards, transparency, etc
- Support non-png input files
