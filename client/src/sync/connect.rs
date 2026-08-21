use super::{ConnStatus, WebsocketBackoff, onmessage_callback};
use crate::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::WebSocket;

impl super::ServerSync {
    pub async fn connect(&mut self) {
        while let WebsocketBackoff::Backoff(exponent) = self.ws {
            match WebSocket::new(&self.url) {
                Ok(ws) => {
                    info!(url = self.url, "connected WS");
                    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
                    let cb = onmessage_callback(self.tx.clone());
                    ws.set_onmessage(Some(cb.as_ref().unchecked_ref()));
                    cb.forget();
                    self.ws = WebsocketBackoff::Connected(ws.clone());
                    self.tx_c.send(ConnStatus::Connected);
                    while ws.ready_state() == 0 {
                        sleep(30).await;
                    }
                    match crate::auth::load_auth_token() {
                        None => error!("not implemented"),
                        Some(auth_token) => {
                            // would it be better to reverse, only mark Connected if this send succeeds?
                            if !self.try_send(&memex_shared::Message::Authorize(auth_token)) {
                               self.ws = WebsocketBackoff::Backoff(0);
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!(?err, "could not open websocket");
                    self.ws = WebsocketBackoff::Backoff(exponent + 1);
                    // max_exponent ~ 68 minutes
                    let timeout = exponential_backoff(500, exponent, 14);
                    debug!(exponent, timeout, "sleeping");
                    sleep(timeout).await;
                }
            }
        }
    }
}

// exponential backoff with jitter;
// uniform distribution between min_ms*2^exponent and min_ms*2^(exponent+1)
fn exponential_backoff(min_ms: u32, exponent: u32, max_exponent: u32) -> i32 {
    let min = (min_ms * 2_u32.pow(std::cmp::min(max_exponent, exponent))) as f64;
    (min + js_sys::Math::random() * min) as i32
}

// https://users.rust-lang.org/t/async-sleep-in-rust-wasm32/78218/4
pub async fn sleep(delay_ms: i32) {
    let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay_ms)
            .unwrap();
    };

    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}
