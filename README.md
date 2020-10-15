# worm-rs

async RESP3 parser for Rust

## Limitations
- Assumes an `Attribute` is always followed by a valid `Value`
- Streaming strings and aggregated data types are not implemented yet
