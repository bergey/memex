use crate::prelude::*;
use automerge::sync as am;
use wasm_bindgen::{JsCast, closure::Closure};

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
    // rx: channel::Receiver<am::Message>,
    tx: Sender<Message>,
}

#[derive(Debug, Clone)]
pub enum WebsocketBackoff {
    Connected(WebSocket),
    Backoff(u32),
}

// exponential backoff with jitter;
// uniform distribution between min_ms*2^exponent and min_ms*2^(exponent+1)
fn exponential_backoff(min_ms: u32, exponent: u32, max_exponent: u32) -> i32 {
    let min = (min_ms * 2_u32.pow(std::cmp::max(max_exponent, exponent))) as f64;
    (min + js_sys::Math::random() * min) as i32
}

// https://users.rust-lang.org/t/async-sleep-in-rust-wasm32/78218/4
async fn sleep(delay_ms: i32) {
    // let delay_ms = delay.millis() as i32;
    let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay_ms);
    };

    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}

impl ServerSync {
    // TODO should init connect?
    pub fn new(
        url: &str,
        // rx: channel::Receiver<am::Message>,
        tx: Sender<Message>,
    ) -> Self {
        ServerSync {
            url: url.to_string(),
            ws: WebsocketBackoff::Backoff(0),
            // rx,
            tx,
        }
    }

    pub async fn connect(&mut self) {
        match WebSocket::new(&self.url) {
            Ok(ws) => {
                ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
                ws.set_onmessage(Some(
                    onmessage_callback(self.tx.clone()).as_ref().unchecked_ref(),
                ));
                self.ws = WebsocketBackoff::Connected(ws);
                self.tx.send(Message::Connected)
            }
            Err(err) => {
                warn!("could not open websocket: {:?}", err);
                let exponent = match self.ws {
                    WebsocketBackoff::Backoff(exponent) => {
                        self.ws = WebsocketBackoff::Backoff(exponent + 1);
                        exponent
                    }
                    WebsocketBackoff::Connected(_) => {
                        self.ws = WebsocketBackoff::Backoff(1);
                        0
                    }
                };
                let timeout = exponential_backoff(500, exponent, 14);
                sleep(timeout).await;
                // TODO try again
            }
        }
    }

    pub fn send(&mut self, message: am::Message) {
        if let WebsocketBackoff::Connected(ws) = &self.ws {
            if ws.send_with_u8_array(&message.encode()).is_err() {
                self.ws = WebsocketBackoff::Backoff(0);
                self.tx.send(Message::Disconnected);
            }
        }
    }
}

fn onmessage_callback(mut tx: Sender<Message>) -> Closure<dyn FnMut(MessageEvent)> {
    Closure::new(move |ev: MessageEvent| {
        on_message_callback_inner(&mut tx, ev);
    })
}

fn on_message_callback_inner(tx: &mut Sender<Message>, ev: MessageEvent) -> Option<()>{
    let abuf = ev.data().dyn_into::<js_sys::ArrayBuffer>().log_error()?;
    let vec = js_sys::Uint8Array::new(&abuf).to_vec();
    let message = am::Message::decode(vec.as_ref()).log_error()?;
    tx.send(Message::Automerge(message));
    Some(())
}
