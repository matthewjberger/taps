use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone)]
pub enum Message<T> {
    Subscribe(String, mpsc::Sender<broadcast::Receiver<T>>),
    Publish(String, T), // Topic and content
}

pub struct Broker<T: Clone> {
    topics: HashMap<String, broadcast::Sender<T>>,
}

impl<T: Clone> Broker<T> {
    pub fn new() -> Self {
        let topics = HashMap::new();
        Self { topics }
    }

    pub async fn run(&mut self, mut worker_rx: mpsc::Receiver<Message<T>>) {
        while let Some(msg) = worker_rx.recv().await {
            match msg {
                Message::Subscribe(topic, tx) => {
                    let broadcast_tx = self
                        .topics
                        .entry(topic.clone())
                        .or_insert_with(|| {
                            let (broadcast_tx, _) = broadcast::channel(32);
                            broadcast_tx
                        })
                        .clone();
                    let _ = tx.send(broadcast_tx.subscribe()).await;
                }
                Message::Publish(topic, content) => {
                    if let Some(broadcast_tx) = self.topics.get(&topic) {
                        let _ = broadcast_tx.send(content);
                    }
                }
            }
        }
    }
}

pub struct Client<T: Clone> {
    broker_tx: mpsc::Sender<Message<T>>,
    subscriptions: HashMap<String, broadcast::Receiver<T>>,
}

impl<T: Clone> Clone for Client<T> {
    fn clone(&self) -> Self {
        Self {
            broker_tx: self.broker_tx.clone(),
            subscriptions: HashMap::new(),
        }
    }
}

impl<T: Clone> Client<T> {
    pub fn new(broker_tx: mpsc::Sender<Message<T>>) -> Self {
        let subscriptions = HashMap::new();
        Self {
            broker_tx,
            subscriptions,
        }
    }

    pub async fn subscribe(&mut self, topic: String) {
        let (tx, mut rx) = mpsc::channel(1);
        let _ = self
            .broker_tx
            .send(Message::Subscribe(topic.clone(), tx))
            .await;
        if let Some(subscription) = rx.recv().await {
            self.subscriptions.insert(topic, subscription);
        }
    }

    pub async fn publish(&self, topic: String, message: T) {
        let _ = self.broker_tx.send(Message::Publish(topic, message)).await;
    }

    pub async fn receive(&mut self, topic: &str) -> Option<T> {
        if let Some(rx) = self.subscriptions.get_mut(topic) {
            rx.recv().await.ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_topic_subscription() {
        let mut broker = Broker::new();
        let (worker_tx, worker_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            broker.run(worker_rx).await;
        });

        let mut client1 = Client::new(worker_tx.clone());
        let mut client2 = Client::new(worker_tx.clone());

        client1.subscribe("topic1".to_string()).await;
        client2.subscribe("topic1".to_string()).await;

        client1
            .publish("topic1".to_string(), "Hello!".to_string())
            .await;
        let msg_from_client2 = client2.receive("topic1").await;

        assert_eq!(msg_from_client2, Some("Hello!".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_topics() {
        let mut broker = Broker::new();
        let (worker_tx, worker_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            broker.run(worker_rx).await;
        });

        let mut client = Client::new(worker_tx.clone());

        client.subscribe("topic1".to_string()).await;
        client.subscribe("topic2".to_string()).await;

        client
            .publish("topic1".to_string(), "Message1".to_string())
            .await;
        client
            .publish("topic2".to_string(), "Message2".to_string())
            .await;

        let msg1 = client.receive("topic1").await;
        let msg2 = client.receive("topic2").await;

        assert_eq!(msg1, Some("Message1".to_string()));
        assert_eq!(msg2, Some("Message2".to_string()));
    }

    #[tokio::test]
    async fn test_no_subscription_no_receive() {
        let mut broker = Broker::new();
        let (worker_tx, worker_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            broker.run(worker_rx).await;
        });

        let mut client = Client::new(worker_tx.clone());

        // Client doesn't subscribe to any topic
        client
            .publish("topic1".to_string(), "Hello!".to_string())
            .await;
        let msg = client.receive("topic1").await;

        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_multiple_clients_multiple_topics() {
        let mut broker = Broker::new();
        let (worker_tx, worker_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            broker.run(worker_rx).await;
        });

        let mut client1 = Client::new(worker_tx.clone());
        let mut client2 = Client::new(worker_tx.clone());
        let mut client3 = Client::new(worker_tx.clone());

        client1.subscribe("topic1".to_string()).await;
        client2.subscribe("topic2".to_string()).await;
        client3.subscribe("topic1".to_string()).await;
        client3.subscribe("topic2".to_string()).await;

        client1
            .publish("topic1".to_string(), "Message1".to_string())
            .await;
        client2
            .publish("topic2".to_string(), "Message2".to_string())
            .await;

        let msg1_client3 = client3.receive("topic1").await;
        let msg2_client3 = client3.receive("topic2").await;

        assert_eq!(msg1_client3, Some("Message1".to_string()));
        assert_eq!(msg2_client3, Some("Message2".to_string()));
    }

    #[tokio::test]
    async fn test_message_ordering() {
        let mut broker = Broker::new();
        let (worker_tx, worker_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            broker.run(worker_rx).await;
        });

        let client1 = Client::new(worker_tx.clone());
        let mut client2 = Client::new(worker_tx.clone());

        client2.subscribe("topic1".to_string()).await;

        client1
            .publish("topic1".to_string(), "Message1".to_string())
            .await;
        sleep(Duration::from_millis(50)).await; // Introducing a delay to ensure ordering
        client1
            .publish("topic1".to_string(), "Message2".to_string())
            .await;

        let msg1 = client2.receive("topic1").await;
        let msg2 = client2.receive("topic1").await;

        assert_eq!(msg1, Some("Message1".to_string()));
        assert_eq!(msg2, Some("Message2".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_clients() {
        let mut broker = Broker::new();
        let (worker_tx, worker_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            broker.run(worker_rx).await;
        });

        let (result_tx, mut result_rx) = mpsc::channel(1);

        let worker_tx1 = worker_tx.clone();

        // Task 1: Client1 subscribes to a topic
        let client1_task = tokio::spawn(async move {
            let mut client1 = Client::new(worker_tx1);
            client1.subscribe("concurrent_topic".to_string()).await;
            let received_msg = client1.receive("concurrent_topic").await;
            let _ = result_tx.send(received_msg).await;
        });

        let worker_tx2 = worker_tx.clone();

        // Task 2: Client2 publishes a message to the same topic
        let client2_task = tokio::spawn(async move {
            let client2 = Client::new(worker_tx2);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await; // Ensure client1 has time to subscribe
            client2
                .publish(
                    "concurrent_topic".to_string(),
                    "Hello from task!".to_string(),
                )
                .await;
        });

        // Await both tasks to complete
        let _ = tokio::try_join!(client1_task, client2_task);

        let received_msg = result_rx
            .recv()
            .await
            .expect("Failed to get result from task");
        assert_eq!(received_msg, Some("Hello from task!".to_string()));
    }
}
