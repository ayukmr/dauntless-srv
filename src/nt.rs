use crate::Data;

use std::{net::TcpStream, thread};
use std::time::Duration;

use std::sync::Arc;
use arc_swap::ArcSwap;

use dauntless::Tag;
use serde::Serialize;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{client::IntoClientRequest, Message, WebSocket};

const UID1: u32 = 16;
const UID2: u32 = 8;

const TYPE1: u32 = 4;
const TYPE2: u32 = 18;

struct NT {
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl NT {
    fn new() -> Self {
        let mut req = "ws://127.0.0.1:5810/nt/dauntless".into_client_request().unwrap();
        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            "v4.1.networktables.first.wpi.edu".parse().unwrap(),
        );

        let (ws, _) = tungstenite::connect(req).unwrap();

        Self { ws }
    }

    fn publish(&mut self, topic: &str, uid: u32, ty: &str) {
        let msg = serde_json::json!([{
            "method": "publish",
            "params": {"name": topic, "pubuid": uid, "type": ty, "properties": {}}
        }]);
        self.ws.send(Message::Text(msg.to_string().into())).unwrap();
    }

    fn send(&mut self, uid: u32, ty: u32, val: impl Serialize) {
        let buf = rmp_serde::to_vec(&(uid, 0i64, ty, val)).unwrap();
        self.ws.send(Message::Binary(buf.into())).ok();
    }
}

pub fn run(state: &Arc<ArcSwap<Data>>) {
    let mut nt = NT::new();

    nt.publish("/dauntless/tags", UID1, "json");
    nt.publish("/dauntless/ids", UID2, "int[]");

    loop {
        let st = state.load();

        let (tags, ids): (Vec<&Tag>, Vec<u32>) =
            st.tags.iter().filter_map(|t| t.id.map(|id| (t, id))).unzip();
        let json = serde_json::to_string(&tags).unwrap();

        nt.send(UID1, TYPE1, json);
        nt.send(UID2, TYPE2, ids);

        thread::sleep(Duration::from_millis(20));
    }
}
