//! # `taps` - Tokio Async Pub/Sub
//!
//! This module provides a generic message broker that allows
//! clients on separate spawned tokio tasks to communicate with each other.
//! The broker is responsible for routing messages between clients based on topics.
//!
//! # Communication Model
//!
//! Clients communicate with the broker through channels provided by the `tokio` library.
//! When a client wishes to send a message or subscribe to a topic, it sends a `Message` to the broker.
//! The broker processes these messages, updates its internal state, and forwards appropriate messages to the intended recipients.
//!
//! # Usage
//!
//!  ```rust
//! use taps::{Broker, Client};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut broker = Broker::new();
//!     let (worker_tx, worker_rx) = tokio::sync::mpsc::channel(32);
//!     tokio::spawn(async move {
//!         broker.run(worker_rx).await;
//!     });
//!
//!    let mut client1 = Client::new(worker_tx.clone());
//!    client1.subscribe("topic1".to_string()).await;
//!
//!    let mut client2 = Client::new(worker_tx.clone());
//!    client2.subscribe("topic1".to_string()).await;
//!
//!    client1
//!       .publish("topic1".to_string(), "Hello from client1!".to_string())
//!      .await;
//!    if let Some(msg_from_client2) = client2.receive("topic1").await {
//!         println!("{msg_from_client2}"); // Outputs: "Hello from client1!"
//!    }
//! }
// ```

mod messaging;

pub use self::messaging::*;
