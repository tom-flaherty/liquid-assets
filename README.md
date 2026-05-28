Rust compression libraries can be found here:

https://crates.io/categories/compression

You can convert a gif to frames using:

`ffmpeg -i mygif.gif frame_%04d.png`

Or

`ffmpeg -i mygif.gif -start_number 1 -vf scale=128:128 frame_%04d.png`

# TODO

- Add another way to build the assets, for people who don't want auto rebuilds
in build.rs
- How to support other display sizes, other RGB standards, transparency, etc
- Add an error generic for the compressor

- There are tonnes of unwraps which may not produce intuivive outputs
- The compression code needs to be split into submodules and subfunctions

- Add MIPIDSI library to the c3 example

- Add postcard serialization so you can embed image width and height into the compressed data?
- Add carful support for turning decompressed data into a embedded_graphics::Image
- You've got a mix of 'graphics' and 'assets'. Change everything to assets
- Give the repo a witty name
- We added REBUILD_GRAPHICS=1 to rebuild the graphics, but this also rebuilds when
the variable changes back to 0. The build.rs file should check that REBUILD_GRAPHICS is
set to 1 before rebuilding the assets.
