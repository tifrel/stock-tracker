use std::sync::Arc;

use actix::prelude::*;
use chrono::prelude::*;
use tokio::sync::mpsc;
use yahoo::{YahooConnector, YahooError};
use yahoo_finance_api as yahoo;

use crate::messages::*;

pub struct Fetcher {
    connector: Arc<YahooConnector>,
    symbols: Vec<String>,
    from: DateTime<Utc>,
    err_tx: mpsc::Sender<YahooError>,
    hist_tx: mpsc::Sender<StockHistory>,
}

impl Fetcher {
    pub fn new(
        symbols: Vec<String>,
        from: DateTime<Utc>,
    ) -> (
        Self,
        mpsc::Receiver<StockHistory>,
        mpsc::Receiver<YahooError>,
    ) {
        let connector = Arc::new(YahooConnector::new());
        let (err_tx, err_rx) = mpsc::channel(64);
        let (hist_tx, hist_rx) = mpsc::channel(symbols.len());
        let fetcher = Self {
            connector,
            symbols,
            from,
            err_tx,
            hist_tx,
        };
        (fetcher, hist_rx, err_rx)
    }
}

impl Actor for Fetcher {
    type Context = Context<Self>;
}

impl Handler<StartFetch> for Fetcher {
    type Result = ();

    fn handle(&mut self, _: StartFetch, _cx: &mut Context<Self>) -> Self::Result {
        let from = self.from;
        let now = Utc::now();

        for symbol in &self.symbols {
            let htx = self.hist_tx.clone();
            let etx = self.err_tx.clone();
            let connector = self.connector.clone();
            let symbol = symbol.to_string();

            actix::spawn(async move {
                let response = connector
                    .get_quote_history_interval(&symbol, from, now, "1d")
                    .await;

                match response {
                    Ok(res) => match res.quotes() {
                        Ok(quotes) => htx
                            .send(StockHistory {
                                symbol,
                                quotes,
                                from,
                            })
                            .await
                            .unwrap(),
                        Err(e) => etx.send(e).await.unwrap(),
                    },
                    Err(e) => etx.send(e).await.unwrap(),
                }
            });
        }
    }
}
