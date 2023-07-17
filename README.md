# tcp-chat-server

`tcp-chat-server` is a basic live chat server over raw TCP, implemented in Rust.

## Running

To run: 

```bash
cargo run
```

When running `tcp-chat-server` exposes one websocket endpoint at `http://localhost:23234`.

## Sending messages

Messages are just strings.

## Testing

When running, test using `netcat 127.0.01 23234`