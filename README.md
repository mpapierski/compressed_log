# compressed_log

A Rust crate designed to allow for easy and efficient remote data collection from IOT devices by plugging
into Rust's logging infrastructure.

Log messages are compressed using ultra lightweight LZ4 compression and then streamed over websockets to
a destination server, where they are dumped to a file for processing. This reduces the transit bandwidth
usage by more than 90% without consuming a large number of CPU cycles.

# Features

- Uses [log](https://crates.io/crates/log) API
- LZ4 to compress log on the fly
- Transmits compressed data over persistent WebSocket connection
- Configurable threshold which will trigger in-memory log rotation and data transmission.

WIP.

# Example

Check out `examples` folder for more examples.

WIP.

Client:

```rust
let level = Level::Info;
let logger = LoggerBuilder::new()
    .set_level(level)
    .set_compression_level(Compression::Slow)
    .set_sink_url("http://127.0.0.1:8000/sink/")
    .build()?;
log::set_boxed_logger(Box::new(logger))?;
log::set_max_level(level.to_level_filter());
```

Server:

WIP.

By default `compressed_log` expects WebSocket endpoint to understand binary messages that contains data compressed with LZ4.
