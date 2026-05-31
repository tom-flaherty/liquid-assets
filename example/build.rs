use liquid_assets_deflate::{Compressor, TargetColorFormat, rebuild_assets_if_changed};

#[allow(unused)]
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

// #[allow(unused)]
// struct BrotlicCompressor {}
// impl Compressor for BrotlicCompressor {
//     type Error = ();

//     fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
//         let mut compressor = brotlic::CompressorWriter::new(Vec::new());
//         compressor.write_all(input_bytes).map_err(|_| ())?;
//         Ok(compressor.into_inner().unwrap())
//     }
// }

// #[allow(unused)]
// struct Lz4FlexCompressor {}
// impl Compressor for Lz4FlexCompressor {
//     type Error = ();

//     fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
//         Ok(lz4_flex::compress_prepend_size(input_bytes))
//     }
// }

// #[allow(unused)]
// struct Lz4Compressor {}
// impl Lz4Compressor {
//     pub fn new() -> Self {
//         Self {}
//     }
// }
// impl Compressor for Lz4Compressor {
//     type Error = ();

//     fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
//         let mut output: Vec<u8> = Vec::new();
//         lz4::EncoderBuilder::new().level(4).build(output).unwrap();
//         Ok(output)
//     }
// }

#[allow(unused)]
struct LzssCompressor<const N: usize> {
    buffer: [u8; 2048],
}
#[allow(unused)]
impl<const N: usize> LzssCompressor<N> {
    pub fn new() -> Self {
        Self {
            buffer: [0_u8; 2048],
        }
    }
}
impl<const N: usize> Compressor for LzssCompressor<N> {
    type Error = lzss::LzssError<void::Void, lzss::SliceWriteError>;

    fn compress(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        use lzss::{SliceReader, SliceWriter};
        let mut output = [0_u8; N];
        type LzssEncoder = lzss::Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;
        let bytes_written = LzssEncoder::compress_with_buffer(
            SliceReader::new(input_bytes),
            SliceWriter::new(&mut output),
            &mut self.buffer,
        )?;
        Ok(output[..bytes_written].to_vec())
    }
}

struct NoCompressor {}
impl Compressor for NoCompressor {
    type Error = ();

    fn compress(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(input_bytes.to_vec())
    }
}

fn main() {
    // let mut compressor = MinizOxideCompressor {};

    // const MAX_INPUT_SIZE: usize = 135 * 240 * 2;
    // let mut compressor: LzssCompressor<MAX_INPUT_SIZE> = LzssCompressor::new();

    let mut compressor = NoCompressor {};

    // let compressor = Lz4Compressor {};
    // let compressor = BrotlicCompressor {};
    // let compressor = Lz4FlexCompressor {};

    rebuild_assets_if_changed(
        "./assets",
        "./asset-binaries",
        TargetColorFormat::Rgb565,
        &mut compressor,
    );

    linker_be_nice();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                what if what.starts_with("_defmt_") => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                what if what.starts_with("esp_rtos_") => {
                    eprintln!();
                    eprintln!(
                        "💡 `esp-radio` has no scheduler enabled. Make sure you have initialized `esp-rtos` or provided an external scheduler."
                    );
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!(
                        "💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests"
                    );
                    eprintln!();
                }
                "free"
                | "malloc"
                | "calloc"
                | "get_free_internal_heap_size"
                | "malloc_internal"
                | "realloc_internal"
                | "calloc_internal"
                | "free_internal" => {
                    eprintln!();
                    eprintln!(
                        "💡 Did you forget the `esp-alloc` dependency or didn't enable the `compat` feature on it?"
                    );
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
