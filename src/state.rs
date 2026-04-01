use crate::{config::Config, meta::Meta};
use crate::data::{self, Data};

use std::ops::Index;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use tokio::sync::Notify;

pub struct States {
    pub states: Vec<Arc<State>>,
    pub meta: Meta,
    notify: Arc<Notify>,
}

impl States {
    pub fn new(n_cams: u32) -> Self {
        let mut next_idx = 0;

        let configs =
            Config::load_all()
                .unwrap_or_else(|_| (0..n_cams).map(|_| {
                    let cfg = Config::default(next_idx);
                    next_idx = cfg.server.camera + 1;
                    cfg
                }).collect());

        let meta = Meta::new(n_cams, &configs);

        let notify = Arc::new(Notify::new());

        let states: Vec<_> =
            (0..n_cams)
                .map(|idx| {
                    let state = Arc::new(State::new(
                        idx,
                        configs[idx as usize],
                        notify.clone(),
                    ));

                    let st = state.clone();
                    thread::spawn(move || data::update(&st));

                    state
                })
                .collect();

        States { states, meta, notify }
    }
}

impl Index<usize> for States {
    type Output = Arc<State>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.states[index]
    }
}

pub struct State {
    pub id: u32,
    pub data: Mutex<Data>,
    config: Mutex<Config>,
    pub notify: Arc<Notify>,
    pub all_notify: Arc<Notify>,
}

impl State {
    pub fn new(id: u32, config: Config, all_notify: Arc<Notify>) -> Self {
        Self {
            id,
            all_notify,
            config: config.into(),
            data: Data::default().into(),
            notify: Notify::new().into(),
        }
    }

    pub fn data(&self) -> MutexGuard<'_, Data> {
        self.data.lock().unwrap()
    }

    pub fn config(&self) -> MutexGuard<'_, Config> {
        self.config.lock().unwrap()
    }
}
