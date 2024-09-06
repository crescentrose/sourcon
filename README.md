# sourcon

Pure Rust async implementation of the [Source RCON protocol](https://developer.valvesoftware.com/wiki/Source_RCON_Protocol).

```rust
use sourcon::client::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let host = "dev.viora.sh:27015";

    // client must be mutable
    let mut client = Client::connect(host, "poop").await?;

    let response = client.command("echo hi").await?;
    assert_eq!(response.body(), "hi");

    Ok(())
}
```

## What is working

* Authentication
* Sending commands to a server
* Receiving responses

## What could be added

* Strongly typed commands instead of arbitrary strings
* Stream UDP logs with password support
* Implement RCON server for testing purposes and Fun
* Tests

## Development

I am not currently actively working on any projects using this library (though it might happen in
the future). If you would like to contribute, please feel free to open issues, submit PRs or even
fork the entire thing.

## License

This project is licensed under the terms of the MIT license. See [LICENSE](LICENSE) for the
full text.
