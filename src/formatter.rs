use chrono::Local;
use log::Record;

/// Function type that accepts a borrowed Record and is supposed to return a string.
pub type Formatter = dyn Fn(&Record) -> String + Sync + Send;

/// Default formatter that is in use when nothing is specified.
pub(crate) fn default_formatter(record: &Record) -> String {
    let timestamp = Local::now();
    format!(
        "{} {:<5} [{}] {}\n",
        timestamp.format("%Y-%m-%d %H:%M:%S"),
        record.level().to_string(),
        record.module_path().unwrap_or_default(),
        record.args()
    )
}
