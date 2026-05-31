Example implementation of the liquid_assets library.

There are example implementations of the Compressor and Decompressor traits - see build.rs and lib.rs. 

The example implementations are for [miniz_oxide](https://crates.io/crates/miniz_oxide), [lzss](https://crates.io/crates/lzss), and for comparison, one with no compression. The lzss crate has a `safe` features, which ensures code safety but decreases performance slightly.

Note that reading memory from flash can take a significant amount of time on embedded hardware. For example, loading a frame with no compression takes around 4.5ms, whereas loading and decompressing with lzss unsafe takes around 6ms. Adding compression to the assets has a surprisingly small impact on performance; this is because flash reading is such a bottleneck, it's only slightly slower to read a smaller amount of data and decompress than to read uncompressed data from flash.

Enable the `display` feature to run an animation loop for [this 1.14" TFT display](https://shop.pimoroni.com/products/adafruit-1-14-240x135-color-newxie-tft-display-st7789?variant=55022898872699) on [this board](https://thepihut.com/products/esp32-c3-devkit-rust-1-4-mb-flash).

| Compression Method | Approx Load + Decompression Time | Total Compressed Data Size |
| ------------------ | -------------------------------- | ---------------------------|
| None               | 4.5ms                            | 725318                     |
| LZSS Safe          | 7ms                              | 121976                     |
| LZSS Unsafe        | 6ms                              | 121976                     |
| Miniz Oxide        | 8ms                              | 61945                      |
