# compressed_log

A Rust crate to compress logs on the fly, and send them over the network after reaching a configured threshold.

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

