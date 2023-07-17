# tcp-chat-server-rs

`tcp-chat-server-rs` is a basic live chat server over raw TCP, implemented in Rust.

## Running

To run: 

```bash
cargo run
```

When running `tcp-chat-server-rs` exposes one websocket endpoint at `http://localhost:23234`.

## Sending messages

Messages are just strings.

## Testing

When running, test using `netcat 127.0.01 23234`