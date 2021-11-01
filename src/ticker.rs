use actix::prelude::*;
use tokio::{sync::mpsc, time};

use crate::messages::*;

pub struct Ticker {
    interval: time::Duration,
    tx_out: mpsc::Sender<StartFetch>,
}

impl Ticker {
    pub fn new(interval: time::Duration, bufsize: usize) -> (Self, mpsc::Receiver<StartFetch>) {
        let (tx_out, rx_out) = mpsc::channel(bufsize);
        (Ticker { interval, tx_out }, rx_out)
    }
}

impl Actor for Ticker {
    type Context = Context<Self>;
}

impl Handler<StartTicking> for Ticker {
    type Result = ();
    fn handle(&mut self, _: StartTicking, _: &mut Context<Self>) {
        let mut interval = time::interval(self.interval);
        let tx_out = self.tx_out.clone();
        actix::spawn(async move {
            loop {
                interval.tick().await;
                let _ = tx_out.send(StartFetch).await;
            }
        });
    }
}
