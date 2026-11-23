use super::{ConnStatus, WebsocketBackoff};

use crate::prelude::*;
use memex_shared::message::Message;

use futures::channel::mpsc as channel;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug)]
pub(super) enum Outcome {
  Success,
  Reconnect,
  Drop,
}

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

  async fn send(&mut self, message: &Message) {
    if let Message::Authorize(_) = message {
      // attempt to connect if we just got a new auth token
      debug!("connect WS to send Authorize");
      self.connect().await; // does it matter if we're already connected?
    }
    let outcome = self.try_send(message);
    debug!(?outcome);
    if let Outcome::Reconnect = outcome {
      // connect if this is the first message we tried to send since a connection error
      self.connect().await;
      let _ = self.try_send(message);
    }
  }

  // true on success
  pub(super) fn try_send(&mut self, message: &Message) -> Outcome {
    match &self.ws {
      WebsocketBackoff::Backoff(_) => {
        return Outcome::Drop;
      }
      WebsocketBackoff::Connected(ws) => {
        if ws.ready_state() > 1 {
          self.disconnected();
          return Outcome::Reconnect;
        }
        if let Err(e) = ws.send_with_u8_array(&message.encode()) {
          debug!(?e, "WS error in send");
          self.disconnected();
          return Outcome::Reconnect;
        }
        return Outcome::Success;
      }
    }
  }
}
