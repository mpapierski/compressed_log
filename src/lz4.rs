pub use lz4::{Encoder, EncoderBuilder};

// A type alias for an LZ4 encoder that writes to memory buffer.
pub type InMemoryEncoder = Encoder<Vec<u8>>;

/// A convenient enum and a syntax sugar to easily specify compression
/// levels.
#[derive(Copy, Clone)]
pub enum Compression {
    /// Equal to "1"
    Fast,
    /// Equal to "9"
    Slow,
    /// Arbitrary level of compression
    Level(u32),
}

impl Compression {
    pub fn to_level(self) -> u32 {
        match self {
            Compression::Fast => 1,
            Compression::Slow => 9,
            Compression::Level(value) => value,
        }
    }
}
