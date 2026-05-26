use asset_compression::{Compressor, TargetColorFormat, rebuild_graphics_if_changed};

struct ZlibCompressor {}

impl Compressor for ZlibCompressor {
    fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, ()> {
        const COMPRESSION_LEVEL: u8 = 5;
        Ok(miniz_oxide::deflate::compress_to_vec(
            input_bytes,
            COMPRESSION_LEVEL,
        ))
    }
}

fn main() {
    // Normally this would go in build.rs
    let zlib_compressor = ZlibCompressor {};
    rebuild_graphics_if_changed(
        "./input",
        "./output",
        TargetColorFormat::Rgb565,
        zlib_compressor,
    )
    .unwrap();
}
