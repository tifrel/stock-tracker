use std::convert::TryFrom;

use chrono::prelude::*;
use yahoo::{YahooConnector, YahooError};
use yahoo_finance_api as yahoo;

use crate::util;

#[derive(Debug)]
pub struct StockHistory {
    symbol: String,
    from: DateTime<Utc>,
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
}

impl StockHistory {
    pub async fn fetch(
        connector: &YahooConnector,
        ticker: &str,
        from: DateTime<Utc>,
    ) -> Result<Self, YahooError> {
        let to = Utc::now();
        let quotes = connector
            .get_quote_history_interval(ticker, from, to, "1d")
            .await?
            .quotes()?;
        let (open, high, low, close) = Self::quote_transpose(quotes);

        Ok(StockHistory {
            symbol: ticker.to_string(),
            from,
            open,
            high,
            low,
            close,
        })
    }

    #[inline]
    fn quote_transpose(quotes: Vec<yahoo::Quote>) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut open = vec![];
        let mut high = vec![];
        let mut low = vec![];
        let mut close = vec![];
        for quote in quotes {
            open.push(quote.open);
            high.push(quote.open);
            low.push(quote.open);
            close.push(quote.open);
        }
        (open, high, low, close)
    }
}

#[derive(Debug)]
pub struct StockInfo {
    symbol: String,
    from: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    sma30: Option<f64>,
}

impl TryFrom<&StockHistory> for StockInfo {
    type Error = ();
    fn try_from(history: &StockHistory) -> Result<Self, <Self as TryFrom<&StockHistory>>::Error> {
        if history.open.len() == 0 {
            return Err(());
        }
        let l = history.open.len();
        let open = history.open[0];
        // unwrap is fine, as we already checked our failure condition above
        let high = util::max(&history.high).unwrap();
        let low = util::min(&history.low).unwrap();
        let close = history.close[l - 1];
        let sma30 = util::avg(&history.open[l - 30..]);
        Ok(Self {
            symbol: history.symbol.clone(),
            from: history.from,
            open,
            high,
            low,
            close,
            sma30,
        })
    }
}

impl StockInfo {
    pub async fn fetch_since(
        connector: &YahooConnector,
        ticker: &str,
        from: DateTime<Utc>,
    ) -> Result<Self, YahooError> {
        // TODO::bench -> we loop over all data while fetching history, and
        // and once more when converting -> should possibly be merged
        let history = StockHistory::fetch(&connector, ticker, from).await?;
        // unwrap should be fine, as we the yahoo api gives us a
        // `YahooError::EmptyDataSet` variant, which is the only reason for
        // the conversion to fail
        Ok(Self::try_from(&history).unwrap())
    }

    pub fn fmt_csv(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{}",
            self.from.to_rfc3339(),
            self.symbol,
            self.open,
            (self.close / self.open - 1.0) * 100.0,
            self.low,
            self.high,
            self.sma30
                .map_or("".to_string(), |sma30| format!("{:.2}", sma30))
        )
    }
}
