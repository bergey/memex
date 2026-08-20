mod connect;

use memex_shared::message::Message;

use crate::prelude::*;
use futures::channel::mpsc as channel;
use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::spawn_local;

use web_sys::{MessageEvent, WebSocket};

#[derive(Debug, Clone, Copy)]
pub enum ConnStatus {
    Connected,
    Disconnected,
}

#[derive(Debug)]
pub struct ServerSync {
    url: String,
    ws: WebsocketBackoff,
    tx: Sender<Message>,
    tx_c: Sender<ConnStatus>,
}

#[derive(Debug, Clone)]
pub enum WebsocketBackoff {
    Connected(WebSocket),
    Backoff(u32),
}

impl ServerSync {
    // spawns a long-lived task
    pub fn start(
        url: String,
    ) -> (
        Sender<Message>,
        channel::Receiver<Message>,
        channel::Receiver<ConnStatus>,
    ) {
        let (tx_up, mut rx_up) = Sender::new(3);
        let (tx_down, rx_down) = Sender::new(10);
        let (tx_c, rx_c) = Sender::new(3);

        let mut handle = Self {
            url,
            ws: WebsocketBackoff::Backoff(0),
            tx: tx_down,
            tx_c,
        };

        spawn_local(async move {
            handle.connect().await;
            loop {
                let msg = rx_up.recv().await.expect("_up channel closed");
                handle.send(&msg).await;
            }
        });

        (tx_up, rx_down, rx_c)
    }

    // no retry limit, but exponential backoff via connect,
    // and when the queue of messages to send fills up, we drop any further
    // so if we finally connect before user restarts, we send the oldest messages,
    // AM ensures sync
    // maybe instead we should drop here if no WS, count on sync-all immediately after connect?
    async fn send(&mut self, message: &Message) {
        while !self.try_send(message) {
            self.connect().await;
        }
    }

    // true on success
    fn try_send(&mut self, message: &Message) -> bool {
        match &self.ws {
            WebsocketBackoff::Backoff(_) => {
                warn!("websocket backoff in send.  should not be reachable");
                return false;
            }
            WebsocketBackoff::Connected(ws) => {
                if ws.ready_state() > 1 {
                    self.disconnected();
                    return false;
                }
                if let Err(e) = ws.send_with_u8_array(&message.encode()) {
                    debug!(?e, "WS error in send");
                    self.disconnected();
                    return false;
                }
                return true; // successful send, done
            }
        }
    }

    fn disconnected(&mut self) {
        self.ws = WebsocketBackoff::Backoff(0);
        self.tx_c.send(ConnStatus::Disconnected);
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
    let message = Message::decode(vec.as_ref()).log_error()?;
    tx.send(message);
    Some(())
}
