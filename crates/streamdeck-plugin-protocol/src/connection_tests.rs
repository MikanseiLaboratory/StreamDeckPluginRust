use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::connection::StreamDeckConnection;
use crate::registration::RegistrationArguments;
use crate::transport::Transport;
use crate::Result;

struct FakeTransport {
    sent: Arc<Mutex<Vec<String>>>,
    incoming: mpsc::UnboundedReceiver<Option<String>>,
    first_send: Option<oneshot::Sender<()>>,
}

#[async_trait]
impl Transport for FakeTransport {
    async fn connect(&mut self, _url: &str) -> Result<()> {
        Ok(())
    }

    async fn send(&mut self, json: &str) -> Result<()> {
        self.sent.lock().unwrap().push(json.to_string());
        if let Some(notify) = self.first_send.take() {
            let _ = notify.send(());
        }
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        Ok(self.incoming.recv().await.flatten())
    }
}

#[tokio::test]
async fn registers_then_dispatches_inbound_events() {
    let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
    let (ready_tx, ready_rx) = oneshot::channel();
    let sent = Arc::new(Mutex::new(Vec::new()));
    let transport = FakeTransport {
        sent: Arc::clone(&sent),
        incoming: incoming_rx,
        first_send: Some(ready_tx),
    };
    let args = RegistrationArguments::parse([
        "-port",
        "12345",
        "-pluginUUID",
        "uuid-1",
        "-registerEvent",
        "registerPlugin",
        "-info",
        "{}",
    ])
    .unwrap();
    let connection = StreamDeckConnection::with_transport(args, transport);
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let run = tokio::spawn(async move {
        connection
            .run(
                |message| {
                    let received_clone = Arc::clone(&received_clone);
                    async move {
                        received_clone.lock().unwrap().push(message.event);
                    }
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
    });

    ready_rx.await.unwrap();
    let register = sent.lock().unwrap()[0].clone();
    assert!(register.contains("registerPlugin"));
    assert!(register.contains("uuid-1"));

    incoming_tx
        .send(Some(r#"{"event":"systemDidWakeUp"}"#.into()))
        .unwrap();
    incoming_tx
        .send(Some(
            r#"{"event":"didReceiveDeepLink","payload":{"url":"ping"}}"#.into(),
        ))
        .unwrap();
    incoming_tx.send(None).unwrap();
    run.await.unwrap();

    assert_eq!(
        *received.lock().unwrap(),
        vec![
            "systemDidWakeUp".to_string(),
            "didReceiveDeepLink".to_string()
        ]
    );
}
