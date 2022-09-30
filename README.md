# Simple TCP game networking example

### [Create streams (Host and Client)](https://github.com/INDA22PlusPlus/net-example/blob/8efe1bf276d70f253f644f9da3b528d593d33d35/src/main.rs#L41-L74)
```rust
// A stream and a boolean indicating wether or not the program is a host or a client
let (stream, client) = {
    let mut args = std::env::args();
    // Skip path to program
    let _ = args.next();

    // Get first argument after path to program
    let host_or_client = args
        .next()
        .expect("Expected arguments: --host or --client 'ip'");

    match host_or_client.as_str() {
        // If the program is running as host we listen on port 8080 until we get a
        // connection then we return the stream.
        "--host" => {
            let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
            (listener.incoming().next().unwrap().unwrap(), false)
        }
        // If the program is running as a client we connect to the specified IP address and
        // return the stream.
        "--client" => {
            let ip = args.next().expect("Expected ip address after --client");
            let stream = TcpStream::connect(ip).expect("Failed to connect to host");
            (stream, true)
        }
        // Only --host and --client are valid arguments
        _ => panic!("Unknown command: {}", host_or_client),
    }
};

// Set TcpStream to non blocking so that we can do networking in the update thread
stream
    .set_nonblocking(true)
    .expect("Failed to set stream to non blocking");

```

### [Recieve packets](https://github.com/INDA22PlusPlus/net-example/blob/8efe1bf276d70f253f644f9da3b528d593d33d35/src/main.rs#L104-L114)
Because the stream is non blocking we can check if data is available and otherwise just return None.
```rust
/// Checks if a move packet is available in returns the new positions otherwise it returns none
fn recieve_move_packet(&mut self) -> Option<(u8, u8)> {
    let mut buf = [0u8; 2];
    match self.stream.read(&mut buf) {
        Ok(_) => Some((buf[0], buf[1])),
        Err(e) => match e.kind() {
            std::io::ErrorKind::WouldBlock => None,
            _ => panic!("Error: {}", e),
        },
    }
}

```

### [Send packets](https://github.com/INDA22PlusPlus/net-example/blob/8efe1bf276d70f253f644f9da3b528d593d33d35/src/main.rs#L116-L124)
```rust
/// Sends a move packet of the current position and sets the state to waiting
fn send_move_packet(&mut self) {
    let mut buf = [self.player_pos.0, self.player_pos.1];
    self.stream
        .write(&mut buf)
        .expect("Failed to send move packet");
    self.state = State::WaitingForOpponent;
}
```
