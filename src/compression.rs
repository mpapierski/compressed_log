pub use flate2::write::ZlibEncoder;
pub use flate2::Compression as FlateCompression;

// A type alias for a encoder that writes to memory buffer.
pub type InMemoryEncoder = ZlibEncoder<Vec<u8>>;

/// A convenient enum and a syntax sugar to easily specify compression
/// levels.
pub type Compression = FlateCompression;


pub fn create_encoder(buffer: Vec<u8>, l: Compression) -> InMemoryEncoder{
     ZlibEncoder::new(buffer, l)
}