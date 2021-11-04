use std::sync::Arc;

use actix::prelude::*;
use chrono::prelude::*;
use tokio::{sync::mpsc, time};
use yahoo::{YahooConnector, YahooError};
use yahoo_finance_api as yahoo;

use crate::messages::*;

pub struct Fetcher {
    connector: Arc<YahooConnector>,
    symbols: Vec<String>,
    from: DateTime<Utc>,
    debounce: time::Duration,
    err_tx: mpsc::Sender<YahooError>,
    hist_tx: mpsc::Sender<StockHistory>,
}

impl Fetcher {
    pub fn new(
        symbols: Vec<String>,
        from: DateTime<Utc>,
        debounce: time::Duration,
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
            debounce,
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
        let debounce = self.debounce;

        for (i, symbol) in self.symbols.iter().enumerate() {
            let mut fetch_spec = FetchSpec {
                from,
                to: now,
                connector: self.connector.clone(),
                symbol: symbol.to_string(),
            };

            let err_tx = self.err_tx.clone();
            let hist_tx = self.hist_tx.clone();
            actix::spawn(async move {
                // debounce, becouse otherwise we get a problem with the API
                // when setting this to 5ms, it fails a lot
                // (`connection to yahoo finance server failed`)
                // set it to 15ms instead, and it mostly passes
                // Problem seems to be related to DNS lookups
                time::sleep(debounce * i as u32).await;

                match fetch_spec.execute().await {
                    Ok(h) => hist_tx.send(h).await.unwrap(),
                    Err(e) => err_tx.send(e).await.unwrap(),
                }
            });
        }
    }
}

struct FetchSpec {
    connector: Arc<YahooConnector>,
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

impl FetchSpec {
    pub async fn execute(&mut self) -> Result<StockHistory, YahooError> {
        let response = self
            .connector
            .get_quote_history_interval(&self.symbol, self.from, self.to, "1d")
            .await;

        match response {
            Ok(res) => match res.quotes() {
                Ok(quotes) => Ok(StockHistory {
                    symbol: self.symbol.clone(),
                    quotes,
                    from: self.from,
                }),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}
