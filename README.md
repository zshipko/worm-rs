# worm-rs

Async RESP3 parser, client and server ported to Rust from [worm-go](https://github.com/zshipko/worm)

## Known limitations
- Ignores all `Attribute` messages
- Streaming strings and aggregated data types are not implemented yet

## Built-in commands
- `HELLO`: simple handshake
- `AUTH`: password base authentication
- `COMMANDS`: list commands
- `PING`: connectivity check

## Examples

### server

The server example is a very basic in memory key-value store

```shell
$ cargo run --example server
```

Then you can connect and execute commands using `redis-cli`:
```shell
$ redis-cli -3 -p 8080 --user test --pass test
127.0.0.1:8080> get something
(nil)
127.0.0.1:8080> set something abc123
OK
127.0.0.1:8080> get something
abc123
127.0.0.1:8080> list
1) something
127.0.0.1:8080> del something
OK
127.0.0.1:8080> list
(nil)
```

### ping

Ping the example server

```shell
$ cargo run --example ping
String("PONG")
```
