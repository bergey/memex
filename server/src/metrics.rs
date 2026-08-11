use prometheus::{
    self, register_histogram, register_int_counter, Histogram, IntCounter,
};

lazy_static! {
    pub static ref WS_CONNECT: IntCounter = register_int_counter!(
        "ws_connect",
        "subtract WS_DISCONNECT to get current active count"
    )
    .unwrap();
    pub static ref WS_DISCONNECT: IntCounter = register_int_counter!(
        "ws_disconnect",
        "subtract from WS_CONNECT to get current active count"
    )
    .unwrap();
    pub static ref WS_SEND_ERROR: IntCounter = register_int_counter!(
        "ws_send_error",
        "number of errors sending to all Websocket connections"
    )
    .unwrap();
    pub static ref AUTOMERGE_MESSAGE_RECEIVED: IntCounter =
        register_int_counter!("automerge_message_received", "number of messages received").unwrap();
    pub static ref AUTOMERGE_MESSAGE_SENT: IntCounter =
        register_int_counter!("automerge_message_sent", "number of messages sent").unwrap();
    pub static ref GARBAGE_MESSAGE_RECEIVED: IntCounter =
        register_int_counter!("garbage_ws_message", "websocket message we ignore").unwrap();
    // pub static ref BROADCAST_QUEUED: IntGauge = register_int_gauge!(
    //     "broadcast_queued",
    //     "number of messages queued in broadcast channel"
    // )
    // .unwrap();
    pub static ref WS_MESSAGE_LATENCY: Histogram =
        register_histogram!("ws_message", "server-side time to process one message").unwrap();
}
