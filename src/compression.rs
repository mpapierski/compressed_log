use crate::logger::{CompressedLogs, PlaintextLogs};
pub use flate2::write::ZlibEncoder;
pub use flate2::Compression as FlateCompression;
use std::io::Error;
use std::io::Write;

pub enum FinishValue {
    Compressed(CompressedLogs),
    Uncompressed(PlaintextLogs),
}

pub struct Encoder {
    compression: Compression,
    encoder: Option<ZlibEncoder<Vec<u8>>>,
    uncompressed_buffer: Vec<String>,
    uncompressed_bytes: usize,
}

impl Encoder {
    pub fn new(level: Compression) -> Encoder {
        let buffer: Vec<u8> = Vec::new();
        match level {
            Compression::None => Encoder {
                compression: level,
                encoder: None,
                uncompressed_buffer: Vec::new(),
                uncompressed_bytes: 0,
            },
            _ => Encoder {
                compression: level,
                encoder: Some(ZlibEncoder::new(buffer, level.into())),
                uncompressed_buffer: Vec::new(),
                uncompressed_bytes: 0,
            },
        }
    }
    pub fn finish(self) -> Result<FinishValue, Error> {
        let l = PlaintextLogs {
            logs: self.uncompressed_buffer.clone(),
        };
        match self.compression {
            Compression::None => {
                let e = FinishValue::Uncompressed(l);
                Ok(e)
            }
            _ => {
                let mut encoder = self.encoder.unwrap();
                let bytes = serde_json::to_vec(&l).unwrap();
                encoder.write_all(&bytes)?;
                let final_bytes = encoder.finish()?;

                // debugging only code to determine compression ratio
                #[cfg(feature = "debug")]
                {
                    let mut total_memory_used = 0usize;
                    for item in self.uncompressed_buffer.iter() {
                        total_memory_used += item.as_bytes().len();
                    }
                    debug_eprintln!(
                        "Compressed {} bytes to {} bytes",
                        total_memory_used,
                        final_bytes.len()
                    );
                }

                let cl = CompressedLogs {
                    compressed_plaintext_logs: final_bytes,
                };
                let e = FinishValue::Compressed(cl);
                Ok(e)
            }
        }
    }
    pub fn len(&self) -> usize {
        self.uncompressed_buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.uncompressed_buffer.is_empty()
    }
    pub fn add_line(&mut self, line: String) {
        self.uncompressed_bytes += line.as_bytes().len();
        self.uncompressed_buffer.push(line);
    }
    pub fn uncompressed_bytes(&mut self) -> usize {
        self.uncompressed_bytes
    }
}

impl From<Compression> for FlateCompression {
    fn from(item: Compression) -> Self {
        match item {
            Compression::Fast => FlateCompression::fast(),
            Compression::Slow => FlateCompression::best(),
            Compression::Suggested => FlateCompression::default(),
            Compression::Level(v) => FlateCompression::new(v),
            Compression::None => FlateCompression::none(),
        }
    }
}

/// A convenient enum and a syntax sugar to easily specify compression
/// levels.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Compression {
    /// Use only Bincode formatting
    None,
    /// Equal to "1"
    Fast,
    /// Equal to "9"
    Slow,
    /// Arbitrary level of compression
    Level(u32),
    /// Suggested by upstream
    Suggested,
}

impl Compression {
    pub fn to_level(self) -> u32 {
        match self {
            Compression::Fast => 1,
            Compression::Slow => 9,
            Compression::Level(value) => value,
            Compression::Suggested => FlateCompression::default().level(),
            Compression::None => 0,
        }
    }
}
