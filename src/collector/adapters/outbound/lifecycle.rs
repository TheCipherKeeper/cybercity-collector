use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Initializing,
    Attesting,
    Active,
    Degraded,
    Locked,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Initializing => write!(f, "initializing"),
            State::Attesting => write!(f, "attesting"),
            State::Active => write!(f, "active"),
            State::Degraded => write!(f, "degraded"),
            State::Locked => write!(f, "locked"),
        }
    }
}

#[derive(Clone)]
pub struct Lifecycle {
    tx: watch::Sender<State>,
    #[allow(dead_code)]
    rx: watch::Receiver<State>,
    tamper_count: Arc<AtomicUsize>,
}

impl Lifecycle {
    pub fn new(initial: State) -> Self {
        let (tx, rx) = watch::channel(initial);
        Self {
            tx,
            rx,
            tamper_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn current(&self) -> State {
        *self.tx.borrow()
    }

    pub fn set(&self, state: State) {
        let old = self.current();
        if old != state {
            info!("lifecycle transition: {} -> {}", old, state);
            let _ = self.tx.send(state);
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<State> {
        self.tx.subscribe()
    }

    pub fn record_tamper(&self) {
        let count = self.tamper_count.fetch_add(1, Ordering::SeqCst) + 1;
        warn!("tamper event recorded (count={})", count);
        if count >= 3 && self.current() != State::Locked {
            self.set(State::Locked);
        } else if self.current() == State::Active {
            self.set(State::Degraded);
        }
    }

    pub fn tamper_count(&self) -> usize {
        self.tamper_count.load(Ordering::SeqCst)
    }
}
