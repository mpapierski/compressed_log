use log::Record;
use time::{format_description, OffsetDateTime};

// /// Function type that accepts a borrowed Record and is supposed to return a string.
pub type Formatter = dyn Fn(&Record) -> String + Sync + Send;

/// Default formatter that is in use when nothing is specified.
pub(crate) fn default_formatter(record: &Record) -> String {
    let format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC ").unwrap();
    let d = OffsetDateTime::now_utc();

    format!(
        "{} {:<5} [{}] {}\n",
        d.format(&format).unwrap(),
        record.level(),
        record.module_path().unwrap_or_default(),
        record.args()
    )
}

#[test]
fn test_date_time_formatting() {
    use time::OffsetDateTime;

    let d = OffsetDateTime::now_utc();
    println!("{:?}", d);

    let format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC ").unwrap();
    println!("{:?}", d.format(&format).unwrap());

    let test = format!(
        "{} {:<5} [{}] {}\n",
        d.format(&format).unwrap(),
        "test",
        "test",
        "test"
    );
    println!("{}", test)
}
