//! Tokio Async Topic-based Pub/Sub Messaging
//!
//! This module provides a generic broker-client communication system where
//! multiple clients can communicate with each other through a central broker.
//! The broker is responsible for routing messages between clients based on topics.
//!
//! # Structures
//!
//! - `Broker<T>`: Represents the central message router. It manages topics and routes messages between subscribers. It is generic over the message type `T`.
//! - `Client<T>`: Represents a client that can subscribe to topics, send, and receive messages through the broker. It is generic over the message type `T`.
//! - `Message<T>`: Represents different types of message actions like subscribing to a topic or publishing content. It is generic over the message type `T`.
//!
//! # Communication Model
//!
//! Clients communicate with the broker through channels provided by the `tokio` library.
//! When a client wishes to send a message or subscribe to a topic, it sends a `Message` to the broker.
//! The broker processes these messages, updates its internal state, and forwards appropriate messages to the intended recipients.
//!
//! # Usage
//!
//! 1. Create an instance of the `Broker<T>`.
//! 2. Create multiple `Client<T>` instances, passing the broker's transmitter channel.
//! 3. Clients can now `publish` messages of type `T` to topics and `receive` messages of type `T` from other clients on subscribed topics.

mod messaging;

pub use self::messaging::*;
