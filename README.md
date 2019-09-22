# compressed_log

[![Latest Version](https://img.shields.io/crates/v/compressed_log.svg)](https://crates.io/crates/compressed_log)
[![Documentation](https://docs.rs/compressed_log/badge.svg)](https://docs.rs/compressed_log)

A Rust crate to compress logs on the fly, and send them over the network after reaching a configured threshold.

This is useful for when you have a fleet of embedded devices running a Rust program, instead of building a complicated
metrics framework you can collect the normal Rust log output securely using https and efficiently using Deflate compression.

The pure rust backend of LibFlate2 is used with only a 100kb in memory buffer. Using the 'fast' compression setting the average
compression ratio in our tests is ~25 without any noticible cpu impact on even embedded MIPS processors.

Right now compressed_log pulls in the full Actix suite to handle futures and async requests. We hope to transition it to native
futures and dramaticly reudce the dependency tree once they become more mature.

On the server side logs are simply dumped into a file, where they can be aggregated and processed by standard log collection
tools.

# Features

- Uses [log](https://crates.io/crates/log) API
- LieFlate2 to compress log on the fly
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
