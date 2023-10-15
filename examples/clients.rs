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
