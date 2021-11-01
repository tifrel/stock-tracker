use actix::prelude::*;
use chrono::prelude::*;
use yahoo_finance_api as yahoo;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct StockInfo {
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub sma30: Option<f64>,
}

impl StockInfo {
    pub fn fmt_csv(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{}",
            self.from.to_rfc3339(),
            self.symbol,
            self.open,
            self.close,
            self.low,
            self.high,
            self.sma30
                .map_or("".to_string(), |sma30| format!("{:.2}", sma30))
        )
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct StockHistory {
    pub symbol: String,
    pub quotes: Vec<yahoo::Quote>,
    pub from: DateTime<Utc>,
}

// signals the fetcher to fetch, starting at DateTime
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartFetch;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartTicking;
