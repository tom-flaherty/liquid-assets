pub trait Compressor {
    fn compress(&self, input_bytes: &[u8]) -> Result<Vec<u8>, ()>;
}
