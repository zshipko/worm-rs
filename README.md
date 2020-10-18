# worm-rs

Async RESP3 parser, client and server ported to Rust from [worm-go](https://github.com/zshipko/worm)

## Known limitations
- Assumes an `Attribute` is always followed by a valid `Value`
- Streaming strings and aggregated data types are not implemented yet
