//! This module demonstrates a combination of Warp's web handling capabilities with a simple
//! RPC (Remote Procedure Call) mechanism using the `taps` library. The main goal is to use Warp
//! to handle HTTP requests, which then trigger RPC operations.
//!
//! # Architecture
//!
//! - **Warp Endpoint**: Accepts HTTP requests. When a request is received, it acts as an RPC Client.
//! - **RPC Server**: Listens for requests, processes them, and sends a response back.
//!
//! Both the Warp Endpoint and the RPC Server use the `taps` library's `Client` type to communicate.
//!
//! # Data Structures
//!
//! - `RequestBody`: Represents the expected structure of the HTTP request's body.
//! - `RpcRequest`: Enum depicting the kinds of requests the RPC server can process.
//! - `RpcResponse`: Enum for the potential responses the RPC server can return.
//! - `RpcMessage`: Encompasses either an `RpcRequest` or an `RpcResponse` along with a unique ID. This ID helps match responses to their requests.
//! - `MessageContent`: Can hold either a request or a response.
//!
//! # Flow
//!
//! 1. The `main` function initializes the broker from the `taps` library.
//! 2. Two tasks are spun up - one represents the Warp Endpoint and the other represents the RPC server.
//! 3. On receiving an HTTP request, the Warp Endpoint sends an RPC request to compute the square of a number.
//! 4. The RPC Server processes this request, sends back the result, and the Warp Endpoint logs the received response.
//!
//! # Testing
//!
//! Once the application is running, you can send HTTP POST requests to `http://127.0.0.1:3030/`. Here's how you can do it for different platforms:
//!
//! ## Windows
//! Use PowerShell:
//! ```powershell
//! $body = @{ data = '' }| ConvertTo-Json
//! Invoke-RestMethod -Uri 'http://127.0.0.1:3030/' -Method Post -Body $body -ContentType 'application/json'
//! ```
//!
//! ## macOS
//! Use `curl`:
//! ```bash
//! curl -H "Content-Type: application/json" -d "{}" http://127.0.0.1:3030/
//! ```
//!
//! The Warp Endpoint will act as an RPC Client, send a request to the RPC Server to compute the square of 5, and the server will respond with the result. The result will then be logged in the console.

use log::{error, info};
use serde::{Deserialize, Serialize};
use taps::{Broker, Client};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::{body, Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Start the message broker
    let mut broker = Broker::<RpcMessage>::new();
    let (broker_tx, broker_rx) = mpsc::channel(32);
    tokio::spawn(async move {
        broker.run(broker_rx).await;
    });

    // Create an rpc client that the rpc server task can use to communicate with the broker
    let rpc_client = Client::<RpcMessage>::new(broker_tx.clone());
    tokio::spawn(run_rpc_server(rpc_client));

    // Create a client that will be used by the warp endpoint handler
    let hello_client = Client::<RpcMessage>::new(broker_tx.clone());

    // Create a warp endpoint for the warp server
    let hello = warp::path::end()
        .and(
            body::json::<RequestBody>()
                .map(Some)
                .or_else(|_err| async { Ok::<(Option<RequestBody>,), Rejection>((None,)) }),
        )
        .and(warp::any().map(move || hello_client.clone()))
        .and_then(handle_request);

    // Start the warp server on the specified address and port.
    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}

// This is the warp endpoint handler
async fn handle_request(
    body: Option<RequestBody>,
    client: Client<RpcMessage>,
) -> Result<Box<dyn Reply>, Rejection> {
    if let Some(_body) = body.as_ref() {
        // If you want to do something with the request body, do it here
    }
    request_rpc(client).await;
    Ok(Box::new(warp::reply::html("Request processed.")))
}

// This uses the client to request an RPC command be executed
async fn request_rpc(mut client: Client<RpcMessage>) {
    // Subscribe to our specific RPC responses
    let request_id = Uuid::new_v4();
    let response_topic = format!("rpc_responses_{}", request_id);
    client.subscribe(response_topic.to_string()).await;

    // Publish an RPC message to execute an RPC command that squares a number
    let request = RpcMessage {
        id: request_id,
        content: MessageContent::Request(RpcRequest::ComputeSquare(5)),
    };
    client.publish("rpc_requests".to_string(), request).await;
    info!("Published a request from warp endpoint.");

    // If we receive a response that is meant for us
    match client.receive(&response_topic).await {
        Some(response) => {
            // Handle it
            handle_rpc_response(response);
        }
        None => error!("Failed to receive a response in warp endpoint."),
    }
}

fn handle_rpc_response(response: RpcMessage) {
    // Figure out what the response is
    match &response.content {
        MessageContent::Response(RpcResponse::SquareResult(value)) => {
            // And handle that response
            info!("Received a response in warp endpoint: {}", value);
        }
        _ => error!("Unexpected message content in warp endpoint."),
    }
}

// The JSON structure that the endpoint should accept
#[derive(Deserialize)]
struct RequestBody {
    _data: String,
}

// These are rpc commands that can be executed by clients
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RpcRequest {
    ComputeSquare(i32),
}

// These are rpc responses that a client may get back
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RpcResponse {
    SquareResult(i32),
}

// The message type sent and received by clients via the broker
#[derive(Clone, Serialize, Deserialize)]
struct RpcMessage {
    // This uuid lets us route the RPC response back to the sender
    id: Uuid,
    content: MessageContent,
}

// Represents the content of the message sent by a client
#[derive(Clone, Serialize, Deserialize)]
enum MessageContent {
    Request(RpcRequest),
    Response(RpcResponse),
}

// This uses a client to subscribe for rpc requests,
// and executes them when it receives one, and then
// a response is then published to the broker.
async fn run_rpc_server(mut rpc_client: Client<RpcMessage>) {
    // Subscribe for rpc requests from other clients
    rpc_client.subscribe("rpc_requests".to_string()).await;
    info!("RPC Server started and subscribed to 'rpc_requests' topic.");

    // Execute the rpc request if we get one
    if let Some(request) = rpc_client.receive("rpc_requests").await {
        execute_rpc_request(request, rpc_client).await;
    } else {
        error!("RPC Server failed to receive a request.");
    }
}

// Executes rpc requests and publishes the result back on the specified client
async fn execute_rpc_request(request: RpcMessage, rpc_client: Client<RpcMessage>) {
    match &request.content {
        MessageContent::Request(RpcRequest::ComputeSquare(value)) => {
            // Simulate some delay or processing time in the server
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Execute the RPC operation, which is squaring a number
            let result = value * value;

            // Publish the response back to the original sender via the UUID
            let response = RpcMessage {
                id: request.id,
                content: MessageContent::Response(RpcResponse::SquareResult(result)),
            };
            let response_topic = format!("rpc_responses_{}", request.id);
            rpc_client.publish(response_topic, response).await;
            info!("RPC Server processed a request and sent a response.");
        }
        _ => error!("Unexpected message content in RPC Server."),
    }
}
