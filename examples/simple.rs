//! This module demonstrates a basic pub-sub (publish-subscribe) mechanism using the `taps` library.
//!
//! The program initializes a broker and spawns two clients. Each client subscribes to the same topic (`topic1`). The first client then publishes a message to this topic, and the second client retrieves and prints the message.
//!
//! # Components
//!
//! - **Broker**: Responsible for managing the channels and distributing messages among clients. It's initialized at the start of the main function.
//! - **Client**: An entity that can both publish messages to topics and subscribe to topics to receive messages. In this module, two clients are created and both subscribe to the same topic.
//!
//! # Flow
//!
//! 1. The main function initializes the broker from the `taps` library and sets up a worker channel for communication.
//! 2. Two client tasks (`client1` and `client2`) are created. Both clients subscribe to `topic1`.
//! 3. `client1` publishes a message "Hello from client1!" to `topic1`.
//! 4. `client2`, which is also subscribed to `topic1`, retrieves the message sent by `client1` and prints it.
//!
//! # Usage
//!
//! Running this module's `main` function will initiate the pub-sub flow, demonstrating basic inter-client communication using the `taps` library.
use taps::{Broker, Client};

#[tokio::main]
async fn main() {
    let mut broker = Broker::new();
    let (worker_tx, worker_rx) = tokio::sync::mpsc::channel(32);
    tokio::spawn(async move {
        broker.run(worker_rx).await;
    });

    let mut client1 = Client::new(worker_tx.clone());
    client1.subscribe("topic1".to_string()).await;

    let mut client2 = Client::new(worker_tx.clone());
    client2.subscribe("topic1".to_string()).await;

    client1
        .publish("topic1".to_string(), "Hello from client1!".to_string())
        .await;
    if let Some(msg_from_client2) = client2.receive("topic1").await {
        println!("{msg_from_client2}"); // Outputs: "Hello from client1!"
    }
}
