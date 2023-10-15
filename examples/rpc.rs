//! This module demonstrates a simple RPC (Remote Procedure Call) setup using the `taps` library.
//!
//! The `taps` library provides a basic pub-sub (publish-subscribe) mechanism, which is used here to facilitate RPC communication.
//!
//! # Architecture
//!
//! - **RPC Client**: Sends a request and waits for a response.
//! - **RPC Server**: Listens for requests, processes them, and sends a response back.
//!
//! Both the client and server use the `taps` library's `Client` type to communicate.
//!
//! # Data Structures
//!
//! - `RpcRequest`: Enum representing the different types of requests the server can handle.
//! - `RpcResponse`: Enum representing the potential responses from the server.
//! - `RpcMessage`: Struct that encapsulates an `RpcRequest` or `RpcResponse` with a unique ID. This ID is used to match responses to their corresponding requests.
//! - `MessageContent`: Enum that can hold either a request or a response.
//!
//! # Flow
//!
//! 1. The main function initializes the broker from the `taps` library.
//! 2. Two tasks are spawned representing the RPC client and the RPC server.
//! 3. The client subscribes to a topic to receive its response. The topic is unique for each client request, derived from a UUID.
//! 4. The client sends an `RpcRequest` wrapped in an `RpcMessage` to the `rpc_requests` topic.
//! 5. The server is subscribed to the `rpc_requests` topic and picks up the client's request.
//! 6. After processing the request, the server sends an `RpcResponse` wrapped in an `RpcMessage` to the response topic specific to the client's request UUID.
//! 7. The client receives the response on its unique topic and processes it.
//!
//! # Usage
//!
//! Running this module's `main` function will initiate the RPC flow. The client sends a request to compute the square of a number, and the server responds with the result.
use log::{error, info};
use taps::{Broker, Client};
use tokio::sync::mpsc;
use uuid::Uuid;

// Define the RPC message structures
#[derive(Debug, Clone)]
enum RpcRequest {
    ComputeSquare(i32),
}

#[derive(Debug, Clone)]
enum RpcResponse {
    SquareResult(i32),
}

#[derive(Clone)]
struct RpcMessage {
    id: Uuid,
    content: MessageContent,
}

#[derive(Clone)]
enum MessageContent {
    Request(RpcRequest),
    Response(RpcResponse),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting the broker...");

    let mut broker = Broker::new();
    let (broker_tx, broker_rx) = mpsc::channel(32);
    let broker_tx = broker_tx.clone();

    tokio::spawn(async move {
        broker.run(broker_rx).await;
    });
    info!("Broker started!");

    let mut worker_client = Client::new(broker_tx.clone());
    let worker_task = tokio::spawn(async move {
        let client_uuid = Uuid::new_v4();
        let response_topic = format!("rpc_responses_{}", client_uuid);

        worker_client.subscribe(response_topic.clone()).await;

        let request = RpcMessage {
            id: client_uuid,
            content: MessageContent::Request(RpcRequest::ComputeSquare(5)),
        };

        worker_client
            .publish("rpc_requests".to_string(), request)
            .await;
        info!("RPC Client sent a request.");

        if let Some(response) = worker_client.receive(&response_topic).await {
            match response.content {
                MessageContent::Response(RpcResponse::SquareResult(value)) => {
                    info!("RPC Client received a response: {}", value);
                }
                _ => error!("Unexpected message content."),
            }
        } else {
            error!("RPC Client failed to receive a response.");
        }
    });

    let mut rpc_client = Client::new(broker_tx.clone());
    let rpc_task = tokio::spawn(async move {
        rpc_client.subscribe("rpc_requests".to_string()).await;
        info!("RPC Server started and subscribed to 'rpc_requests' topic.");

        if let Some(request) = rpc_client.receive("rpc_requests").await {
            match request.content {
                MessageContent::Request(RpcRequest::ComputeSquare(value)) => {
                    // Simulate some delay or processing time in the server
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    let result = value * value;
                    let response = RpcMessage {
                        id: request.id,
                        content: MessageContent::Response(RpcResponse::SquareResult(result)),
                    };
                    let response_topic = format!("rpc_responses_{}", request.id);
                    rpc_client.publish(response_topic, response).await;
                    info!("RPC Server processed a request and sent a response.");
                }
                _ => error!("Unexpected message content."),
            }
        } else {
            error!("RPC Server failed to receive a request.");
        }
    });

    let _ = tokio::join!(worker_task, rpc_task);
}
