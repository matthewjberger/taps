//! This module demonstrates a basic pub-sub (publish-subscribe) pattern using the `taps` library with logging capabilities.
//!
//! The primary components of this module include a broker, a publisher client, and a subscriber client. The publisher client continuously publishes messages to a topic named "news", while the subscriber client continuously listens for and logs these messages.
//!
//! # Components
//!
//! - **Broker**: The core message-passing component responsible for managing topics and distributing messages among clients. It's initialized at the start of the main function.
//!
//! - **Publisher Client**: A client that continuously sends messages to the "news" topic at regular intervals.
//!
//! - **Subscriber Client**: A client that subscribes to the "news" topic and logs any messages it receives.
//!
//! # Flow
//!
//! 1. The main function initializes the `env_logger` to enable logging.
//! 2. The broker from the `taps` library is initialized and a worker channel is set up for inter-process communication.
//! 3. The publisher client task is spawned. This client sends a new message to the "news" topic every 2 seconds.
//! 4. The subscriber client task is spawned. This client listens for messages on the "news" topic and logs them.
//! 5. The main function awaits the completion of both client tasks (though they are designed to run indefinitely).
//!
//! # Logging
//!
//! This module makes use of the `log` crate to provide detailed logging of its operations. This includes starting the broker, sending messages from the publisher, and receiving messages with the subscriber. Proper error logging is also implemented for any potential failures in message reception.
//!
//! # Usage
//!
//! Running this module's `main` function will initiate the pub-sub flow, demonstrating inter-client communication with real-time logging using the `taps` library.
use log::{error, info};
use std::time::Duration;
use taps::{Broker, Client};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting the broker...");

    let mut broker = Broker::new();
    let (broker_tx, broker_rx) = mpsc::channel(32);
    let sub_broker_tx = broker_tx.clone();

    tokio::spawn(async move {
        broker.run(broker_rx).await;
    });
    info!("Broker started!");

    let publisher_task = tokio::spawn(async move {
        let publisher = Client::new(broker_tx.clone());
        info!("Publisher client started.");
        loop {
            publisher
                .publish("news".to_string(), "Latest news update!".to_string())
                .await;
            info!("Publisher sent a new message.");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    let subscriber_task = tokio::spawn(async move {
        let mut subscriber = Client::new(sub_broker_tx.clone());
        subscriber.subscribe("news".to_string()).await;
        info!("Subscriber client started and subscribed to 'news' topic.");
        loop {
            if let Some(msg) = subscriber.receive("news").await {
                info!("Subscriber received: {}", msg);
            } else {
                error!("Subscriber failed to receive a message.");
            }
        }
    });

    let _ = tokio::join!(publisher_task, subscriber_task);
}
