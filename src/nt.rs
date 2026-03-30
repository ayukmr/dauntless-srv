use crate::data::{CameraTag, St};

use std::sync::Arc;
use std::thread;
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
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
    delta: i64,
}

impl NT {
    fn new() -> Result<Self> {
        let mut req = HOST.into_client_request().unwrap();
        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            "v4.1.networktables.first.wpi.edu".parse().unwrap(),
        );

        let (mut ws, _) = tungstenite::connect(req)?;

        let local = now();
        let buf = rmp_serde::to_vec(&(-1i64, 0i64, 2u32, local)).unwrap();

        ws.send(Message::Binary(buf.into()))?;
        let msg = ws.read()?;

        let (_, server, _, t0): (i64, i64, u32, i64) = rmp_serde::from_slice(&msg.into_data())?;
        let t1 = now();

        let rtt = t1 - t0;
        let delta = server + rtt / 2 - t0;

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
        let buf = rmp_serde::to_vec(&(uid, now() + self.delta, ty, val)).unwrap();
        self.ws.send(Message::Binary(buf.into()))?;
        Ok(())
    }
}

pub fn run(states: Vec<Arc<St>>) {
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
            if tick(&mut nt, &states).is_err() {
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

fn tick(nt: &mut NT, states: &Vec<Arc<St>>) -> Result<()> {
    let (s_tags, times): (Vec<Vec<CameraTag>>, Vec<SystemTime>) =
        states
            .iter()
            .map(|st| {
                let data = st.data();
                (data.tags.clone(), data.time)
            })
            .unzip();

    let (tags, ids): (Vec<CameraTag>, Vec<u32>) =
        s_tags
            .iter()
            .flatten()
            .filter_map(|t| t.tag.id.map(|id| (t, id)))
            .unzip();

    let json = serde_json::to_string(&tags).unwrap();

    let time = times.iter().max().unwrap();

    nt.send(UID1, TYPE1, json)?;
    nt.send(UID2, TYPE2, ids)?;
    nt.send(UID3, TYPE3, time.duration_since(UNIX_EPOCH)?.as_secs_f64())?;

    Ok(())
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64
}
