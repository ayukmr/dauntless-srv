use crate::Data;

use dauntless::Tag;

use anyhow::Result;

use std::thread;
use std::net::TcpStream;
use std::time::{Duration, UNIX_EPOCH};

use std::sync::Arc;
use arc_swap::ArcSwap;

use colored::Colorize;

use serde::Serialize;

use tungstenite::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::client::IntoClientRequest;

const HOST: &str = "ws://10.49.4.2:5810/nt/dauntless";

const UID1: u32 = 16;
const UID2: u32 = 8;
const UID3: u32 = 4;

const TYPE1: u32 = 4;
const TYPE2: u32 = 18;
const TYPE3: u32 = 1;

struct NT {
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl NT {
    fn new() -> Result<Self> {
        let mut req = HOST.into_client_request().unwrap();
        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            "v4.1.networktables.first.wpi.edu".parse().unwrap(),
        );

        let (ws, _) = tungstenite::connect(req)?;

        Ok(Self { ws })
    }

    fn publish(&mut self, topic: &str, uid: u32, ty: &str) -> Result<()> {
        let msg = serde_json::json!([{
            "method": "publish",
            "params": {
                "name": topic,
                "pubuid": uid,
                "type": ty,
                "properties": {},
            },
        }]);
        self.ws.send(Message::Text(msg.to_string().into()))?;
        Ok(())
    }

    fn send(&mut self, uid: u32, ty: u32, val: impl Serialize) -> Result<()> {
        let buf = rmp_serde::to_vec(&(uid, 0i64, ty, val)).unwrap();
        self.ws.send(Message::Binary(buf.into()))?;
        Ok(())
    }
}

pub fn run(state: &Arc<ArcSwap<Data>>) {
    loop {
        let mut nt = loop {
            match init() {
                Ok(nt) => break nt,
                Err(_) => {
                    println!("\rnt: {}", "init failed".red());
                    thread::sleep(Duration::from_millis(2000));
                }
            }
        };

        println!("\rnt: {}", "connected".green());

        loop {
            if tick(&mut nt, state).is_err() {
                println!("\rnt: {}", "tick failed".red());
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn init() -> Result<NT> {
    let mut nt = NT::new()?;

    nt.publish("/dauntless/tags", UID1, "json")?;
    nt.publish("/dauntless/ids", UID2, "int[]")?;
    nt.publish("/dauntless/time", UID3, "double")?;

    Ok(nt)
}

fn tick(nt: &mut NT, state: &Arc<ArcSwap<Data>>) -> Result<()> {
    let st = state.load();

    let (tags, ids): (Vec<&Tag>, Vec<u32>) =
        st.tags.iter().filter_map(|t| t.id.map(|id| (t, id))).unzip();

    let json = serde_json::to_string(&tags).unwrap();

    nt.send(UID1, TYPE1, json)?;
    nt.send(UID2, TYPE2, ids)?;
    nt.send(UID3, TYPE3, st.time.duration_since(UNIX_EPOCH)?.as_millis() as f32 / 1000.0)?;

    Ok(())
}
