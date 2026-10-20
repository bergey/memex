use super::{ConnStatus, WebsocketBackoff};

use crate::prelude::*;
use memex_shared::message::Message;

use futures::channel::mpsc as channel;
use wasm_bindgen_futures::spawn_local;

impl super::ServerSync {
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
  pub (super) fn try_send(&mut self, message: &Message) -> bool {
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
