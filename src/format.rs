///! Implementation of the record formatting logic.
///
/// It defines the way an existing Record is formatted
/// into bytes.
use chrono::{DateTime, Local};
use log::Record;
use std::io;

/// A trait that defines operations on the formatter.
///
/// A LogFormat instance can be created with a borrowed
/// Record, and can be serialized to any object that implements
/// io::Write.
pub trait LogFormat<'a, 'r: 'a> {
    /// Create an instance of the record
    fn from_record(record: &'a Record<'r>) -> Self;
    /// Serialize a LogFormat into bytes
    fn serialize<T: io::Write>(&self, output: &mut T) -> io::Result<usize>;
}

/// BinaryLogFormat serializes the data as a binary string.
pub struct BinaryLogFormat<'a, 'r: 'a> {
    record: &'a Record<'r>,
    /// A timestamp kept at the formatter creation date
    timestamp: DateTime<Local>,
}

impl<'a, 'r: 'a> LogFormat<'a, 'r> for BinaryLogFormat<'a, 'r> {
    fn from_record(record: &'a Record<'r>) -> Self {
        Self {
            record,
            // Timestamp is acquired at the time of LogFormat creation,
            // as LogFormat instances could be buffered.
            timestamp: Local::now(),
        }
    }
    fn serialize<T: io::Write>(&self, output: &mut T) -> io::Result<usize> {
        // TODO: We may want to do something more sophisticated like serializing
        // log Record into a structured binary format (i.e. protobuf), but for
        // a starter lets just dump it as a string and the delimiter would be
        // a new line character.
        let data = format!(
            "{} {:<5} [{}] {}\n",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.record.level().to_string(),
            self.record.module_path().unwrap_or_default(),
            self.record.args()
        );
        output.write(data.as_bytes())
    }
}

#[test]
fn binary() {
    use log::{Level, Record};
    let record = Record::builder()
        .args(format_args!("Error!"))
        .level(Level::Error)
        .target("myApp")
        .file(Some("server.rs"))
        .line(Some(144))
        .module_path(Some("server"))
        .build();
    let formatted = BinaryLogFormat::from_record(&record);
    let mut vec = Vec::new();
    formatted.serialize(&mut vec).unwrap();
    assert!(String::from_utf8(vec)
        .unwrap()
        .ends_with(" ERROR [server] Error!\n"));
}
