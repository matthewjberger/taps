# Taps

[<img alt="github" src="https://img.shields.io/badge/github-matthewjberger/taps-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/matthewjberger/taps)
[<img alt="crates.io" src="https://img.shields.io/crates/v/taps.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/taps)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-taps-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/taps)

`taps` is a `Topic-based Asynchronous Pub/Sub broker` that allows multiple clients to communicate with each other through a central broker, routing messages based on topics.

## Features

- **Topic-based Routing**: Easily send and receive messages based on topics.
- **Asynchronous**: Built for asynchronous environments, utilizing the power of Tokio.
- **Scalable**: Designed to handle multiple clients communicating simultaneously.

## Quick Start

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
taps = "0.1.0"
```

### Usage

```rust
use taps::{Broker, Client};

#[tokio::main]
async fn main() {
    let mut broker = Broker::new();
    let (worker_tx, worker_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        broker.run(worker_rx).await;
    });

    let mut client1 = Client::new(worker_tx.clone());
    let mut client2 = Client::new(worker_tx.clone());

    client1.subscribe("topic1".to_string()).await;
    client2.subscribe("topic1".to_string()).await;

    client1.publish("topic1".to_string(), "Hello from client1!".to_string()).await;
    let msg_from_client2 = client2.receive("topic1").await;
    println!("{:?}", msg_from_client2); // Outputs: Some("Hello from client1!")
}
```
## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for details.

---

You might want to customize some parts, especially if you decide to add contribution guidelines, a logo, or any other additional information.