# compressed_log

A Rust crate to compress logs on the fly, and send them over the network after reaching a configured threshold.

This is useful for when you have a fleet of embedded devices running a Rust program, instead of building a complicated
metrics framework you can collect the normal Rust log output securely using https and efficiently using LZ4 compression.

LZ4 is both fast enough to run on even the most resource starved devices and effective enough to reduce log size by 20x
or more from naive byte streams.

On the server side logs are simply dumped into a file, where they can be aggregated and processed by standard log collection
tools.

# Features

- Uses [log](https://crates.io/crates/log) API
- LZ4 to compress log on the fly
- Transmits compressed data over persistent WebSocket connection
- Configurable threshold which will trigger in-memory log rotation and data transmission.
- Fully architecture portable, including Mips and other BE architectures

# Example

Check out `examples` folder for more examples. Including custom formats for collected logs.

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

The current server reference implementaiton is [compressed_log_sink](https://github.com/althea-mesh/compressed_log_sink)
