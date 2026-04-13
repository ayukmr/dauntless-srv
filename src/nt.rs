use crate::data::CameraTag;
use crate::state::State;

use std::sync::Arc;
use std::thread;
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

use tokio::sync::Notify;
use tungstenite::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::client::IntoClientRequest;

const HOST: &str = "ws://10.49.4.2:5810/nt/dauntless";

const UID_JSON: u32 = 16;
const UID_IDS: u32 = 8;
const UID_TIME: u32 = 4;

const TYPE_JSON: u32 = 4;
const TYPE_INTLIST: u32 = 18;
const TYPE_DOUBLE: u32 = 1;
const TYPE_INT: u32 = 2;

struct NT {
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
    delta: i64,
}

impl NT {
    fn new() -> Result<Self> {
        let mut req = HOST.into_client_request()?;
        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            "v4.1.networktables.first.wpi.edu".parse()?,
        );

        let (mut ws, _) = tungstenite::connect(req)?;

        let local = now();
        let buf = rmp_serde::to_vec(&(-1i64, 0i64, TYPE_INT, local))?;

        ws.send(Message::Binary(buf.into()))?;
        let msg = ws.read()?;

        let (_, server, _, t0): (i64, i64, u32, i64) = rmp_serde::from_slice(&msg.into_data())?;
        let t1 = now();

        let tt = (t1 - t0) / 2;
        let delta = server - (t1 - tt);

        Ok(Self { ws, delta })
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
        let buf = rmp_serde::to_vec(&(uid, now() + self.delta as i64, ty, val))?;
        self.ws.send(Message::Binary(buf.into()))?;

        Ok(())
    }
}

pub async fn run(states: Vec<Arc<State>>, notify: Arc<Notify>) {
    loop {
        let mut nt = loop {
            match init() {
                Ok(nt) => break nt,
                Err(err) => {
                    println!("\rnt: {} [reason: {}]", "init failed".red(), err);
                    thread::sleep(Duration::from_millis(2000));
                }
            }
        };

        println!("\rnt: {}", "connected".green());

        loop {
            if let Err(err) = tick(&mut nt, &states) {
                println!("\rnt: {} [reason: {}]", "tick failed".red(), err);
                break;
            }
            notify.notified().await;
        }
    }
}

fn init() -> Result<NT> {
    let mut nt = NT::new()?;

    nt.publish("/dauntless/tags", UID_JSON, "json")?;
    nt.publish("/dauntless/ids", UID_IDS, "int[]")?;
    nt.publish("/dauntless/time", UID_TIME, "double")?;

    Ok(nt)
}

fn tick(nt: &mut NT, states: &[Arc<State>]) -> Result<()> {
    let (tags, ids): (Vec<CameraTag>, Vec<u32>) =
        states
            .iter()
            .flat_map(|st| st.data().tags.clone())
            .filter_map(|t| {
                t.tag.id.map(|id| {
                    let mut tag = t.clone();
                    tag.time += nt.delta as f64 / 1_000_000.0;
                    (tag, id)
                })
            })
            .unzip();

    let json = serde_json::to_string(&tags)?;

    nt.send(UID_JSON, TYPE_JSON, json)?;
    nt.send(UID_IDS, TYPE_INTLIST, ids)?;
    nt.send(UID_TIME, TYPE_DOUBLE, SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64())?;

    Ok(())
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64
}
