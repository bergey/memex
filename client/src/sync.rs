mod connect;

use crate::prelude::*;
use automerge::sync as am;
use futures::channel::mpsc as channel;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;

use web_sys::{MessageEvent, WebSocket};

// messages from the Websocket task to the Library task
// in the opposite direction, Library -> Websocket is just sync::Message
#[derive(Debug, Clone)]
pub enum Message {
    // TODO LibraryId, other message types
    Automerge(am::Message),
    Connected,
    Disconnected,
}

#[derive(Debug)]
pub struct ServerSync {
    url: String,
    ws: WebsocketBackoff,
    tx: Sender<Message>,
}

#[derive(Debug, Clone)]
pub enum WebsocketBackoff {
    Connected(WebSocket),
    Backoff(u32),
}

impl WebsocketBackoff {
    pub fn is_not_connected(&self) -> bool {
        match self {
            WebsocketBackoff::Connected(_) => false,
            WebsocketBackoff::Backoff(_) => true,
        }
    }
}

impl ServerSync {
    // spawns a long-lived task
    pub fn start(url: String) -> (Sender<am::Message>, channel::Receiver<Message>) {
        let (tx_up, mut rx_up) = Sender::new(3);
        let (tx_down, rx_down) = Sender::new(10);
        let mut handle = Self {
            url,
            ws: WebsocketBackoff::Backoff(0),
            tx: tx_down,
        };

        spawn_local(async move {
            handle.connect().await;
            loop {
                let msg = rx_up.recv().await.expect("_up channel closed");
                handle.send(msg).await;
            }
        });

        (tx_up, rx_down)
    }

    async fn send(&mut self, message: am::Message) {
        if let WebsocketBackoff::Connected(ws) = &self.ws {
            if ws.send_with_u8_array(&message.encode()).is_err() {
                self.ws = WebsocketBackoff::Backoff(0);
                self.tx.send(Message::Disconnected);
                self.connect().await
            }
        }
    }
}

fn onmessage_callback(mut tx: Sender<Message>) -> Closure<dyn FnMut(MessageEvent)> {
    Closure::new(move |ev: MessageEvent| {
        on_message_callback_inner(&mut tx, ev);
    })
}

fn on_message_callback_inner(tx: &mut Sender<Message>, ev: MessageEvent) -> Option<()> {
    let abuf = ev.data().dyn_into::<js_sys::ArrayBuffer>().log_error()?;
    let vec = js_sys::Uint8Array::new(&abuf).to_vec();
    let message = am::Message::decode(vec.as_ref()).log_error()?;
    tx.send(Message::Automerge(message));
    Some(())
}
