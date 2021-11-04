// Most of this code is simply copied from `yahoo_finance_api`. I was trying to
// find out why so many of my requests would fail, and it turns out to be

use chrono::prelude::*;
use reqwest::StatusCode;

mod quotes {
    use std::collections::HashMap;

    use serde::Deserialize;

    use super::Error;

    #[derive(Deserialize, Debug)]
    pub struct YResponse {
        pub chart: YChart,
    }

    impl YResponse {
        fn check_consistency(&self) -> Result<(), Error> {
            for stock in &self.chart.result {
                let n = stock.timestamp.len();
                if n == 0 {
                    return Err(Error::EmptyDataSet);
                }
                let quote = &stock.indicators.quote[0];
                if quote.open.len() != n
                    || quote.high.len() != n
                    || quote.low.len() != n
                    || quote.volume.len() != n
                    || quote.close.len() != n
                {
                    return Err(Error::DataInconsistency);
                }
                if let Some(ref adjclose) = stock.indicators.adjclose {
                    if adjclose[0].adjclose.len() != n {
                        return Err(Error::DataInconsistency);
                    }
                }
            }
            Ok(())
        }

        pub fn from_json(json: serde_json::Value) -> Result<YResponse, Error> {
            serde_json::from_value(json).map_err(|e| Error::DeserializeFailed(e.to_string()))
        }

        /// Return the latest valid quote
        pub fn last_quote(&self) -> Result<Quote, Error> {
            self.check_consistency()?;
            let stock = &self.chart.result[0];
            let n = stock.timestamp.len();
            for i in (0..n).rev() {
                let quote = stock.indicators.get_ith_quote(stock.timestamp[i], i);
                if quote.is_ok() {
                    return quote;
                }
            }
            Err(Error::EmptyDataSet)
        }

        pub fn quotes(&self) -> Result<Vec<Quote>, Error> {
            self.check_consistency()?;
            let stock = &self.chart.result[0];
            let mut quotes = Vec::new();
            let n = stock.timestamp.len();
            for i in 0..n {
                let timestamp = stock.timestamp[i];
                let quote = stock.indicators.get_ith_quote(timestamp, i);
                if let Ok(q) = quote {
                    quotes.push(q);
                }
            }
            Ok(quotes)
        }

        /// This method retrieves information about the splits that might have
        /// occured during the considered time period
        pub fn splits(&self) -> Result<Vec<Split>, Error> {
            self.check_consistency()?;
            let stock = &self.chart.result[0];
            if let Some(events) = &stock.events {
                if let Some(splits) = &events.splits {
                    let mut data = splits.values().cloned().collect::<Vec<Split>>();
                    data.sort_unstable_by_key(|d| d.date);
                    return Ok(data);
                }
            }
            Ok(vec![])
        }
        /// This method retrieves information about the dividends that have
        /// been recorded during the considered time period.
        ///
        /// Note: Date is the ex-dividend date)
        pub fn dividends(&self) -> Result<Vec<Dividend>, Error> {
            self.check_consistency()?;
            let stock = &self.chart.result[0];
            if let Some(events) = &stock.events {
                if let Some(dividends) = &events.dividends {
                    let mut data = dividends.values().cloned().collect::<Vec<Dividend>>();
                    data.sort_unstable_by_key(|d| d.date);
                    return Ok(data);
                }
            }
            Ok(vec![])
        }
    }

    /// Struct for single quote
    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub struct Quote {
        pub timestamp: u64,
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub volume: u64,
        pub close: f64,
        pub adjclose: f64,
    }

    #[derive(Deserialize, Debug)]
    pub struct YChart {
        pub result: Vec<YQuoteBlock>,
        pub error: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct YQuoteBlock {
        pub meta: YMetaData,
        pub timestamp: Vec<u64>,
        pub events: Option<EventsBlock>,
        pub indicators: QuoteBlock,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct YMetaData {
        pub currency: String,
        pub symbol: String,
        pub exchange_name: String,
        pub instrument_type: String,
        pub first_trade_date: i32,
        pub regular_market_time: u32,
        pub gmtoffset: i32,
        pub timezone: String,
        pub exchange_timezone_name: String,
        pub regular_market_price: f64,
        pub chart_previous_close: f64,
        #[serde(default)]
        pub previous_close: Option<f64>,
        #[serde(default)]
        pub scale: Option<i32>,
        pub price_hint: i32,
        pub current_trading_period: TradingPeriod,
        #[serde(default)]
        pub trading_periods: Option<Vec<Vec<PeriodInfo>>>,
        pub data_granularity: String,
        pub range: String,
        pub valid_ranges: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct TradingPeriod {
        pub pre: PeriodInfo,
        pub regular: PeriodInfo,
        pub post: PeriodInfo,
    }

    #[derive(Deserialize, Debug)]
    pub struct PeriodInfo {
        pub timezone: String,
        pub start: u32,
        pub end: u32,
        pub gmtoffset: i32,
    }

    #[derive(Deserialize, Debug)]
    pub struct QuoteBlock {
        quote: Vec<QuoteList>,
        #[serde(default)]
        adjclose: Option<Vec<AdjClose>>,
    }

    impl QuoteBlock {
        fn get_ith_quote(&self, timestamp: u64, i: usize) -> Result<Quote, Error> {
            let adjclose = match &self.adjclose {
                Some(adjclose) => adjclose[0].adjclose[i],
                None => None,
            };
            let quote = &self.quote[0];
            // reject if close is not set
            if quote.close[i].is_none() {
                return Err(Error::EmptyDataSet);
            }
            Ok(Quote {
                timestamp,
                open: quote.open[i].unwrap_or(0.0),
                high: quote.high[i].unwrap_or(0.0),
                low: quote.low[i].unwrap_or(0.0),
                volume: quote.volume[i].unwrap_or(0),
                close: quote.close[i].unwrap(),
                adjclose: adjclose.unwrap_or(0.0),
            })
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct AdjClose {
        adjclose: Vec<Option<f64>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct QuoteList {
        pub volume: Vec<Option<u64>>,
        pub high: Vec<Option<f64>>,
        pub close: Vec<Option<f64>>,
        pub low: Vec<Option<f64>>,
        pub open: Vec<Option<f64>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct EventsBlock {
        pub splits: Option<HashMap<u64, Split>>,
        pub dividends: Option<HashMap<u64, Dividend>>,
    }

    /// This structure simply models a split that has occured.
    #[derive(Deserialize, Debug, Clone)]
    pub struct Split {
        /// This is the date (timestamp) when the split occured
        pub date: u64,
        /// Numerator of the split. For instance a 1:5 split means you get 5 share
        /// wherever you had one before the split. (Here the numerator is 1 and
        /// denom is 5). A reverse split is considered as nothing but a regular
        /// split with a numerator > denom.
        pub numerator: u64,
        /// Denominator of the split. For instance a 1:5 split means you get 5 share
        /// wherever you had one before the split. (Here the numerator is 1 and
        /// denom is 5). A reverse split is considered as nothing but a regular
        /// split with a numerator > denom.
        pub denominator: u64,
        /// A textual representation of the split.
        #[serde(rename = "splitRatio")]
        pub split_ratio: String,
    }

    /// This structure simply models a dividend which has been recorded.
    #[derive(Deserialize, Debug, Clone)]
    pub struct Dividend {
        /// This is the price of the dividend
        pub amount: f64,
        /// This is the ex-dividend date
        pub date: u64,
    }
}

use quotes::*;

#[derive(Debug)]
pub enum Error {
    ConnectionFailed(reqwest::Error),
    InvalidJson,
    FetchFailed(String),
    EmptyDataSet,
    DataInconsistency,
    DeserializeFailed(String),
}

async fn send_request(url: &str) -> Result<serde_json::Value, Error> {
    use async_compat::CompatExt;
    let resp = reqwest::get(url).compat().await;
    if resp.is_err() {
        // eprintln!("{:?}", resp);
        return Err(Error::ConnectionFailed(resp.unwrap_err()));
    }
    let resp = resp.unwrap();
    match resp.status() {
        StatusCode::OK => resp.json().await.map_err(|_| Error::InvalidJson),
        status => Err(Error::FetchFailed(format!("Status Code: {}", status))),
    }
}

macro_rules! YCHART_PERIOD_QUERY {
    () => {
        "{url}/{symbol}?symbol={symbol}&period1={start}&period2={end}&interval={interval}&events=div|split"
    };
}

async fn get_quote_history_interval(
    ticker: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    interval: &str,
) -> Result<YResponse, Error> {
    let url = format!(
        YCHART_PERIOD_QUERY!(),
        url = "https://query1.finance.yahoo.com/v8/finance/chart",
        symbol = ticker,
        start = start.timestamp(),
        end = end.timestamp(),
        interval = interval
    );
    YResponse::from_json(send_request(&url).await?)
}

struct FetchSpec {
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

impl FetchSpec {
    pub async fn execute(&mut self) -> Result<(), Error> {
        let response = get_quote_history_interval(&self.symbol, self.from, self.to, "1d").await;

        match response {
            Ok(res) => match res.quotes() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

#[tokio::main]
async fn main() {
    let from = Utc.ymd(2021, 9, 1).and_hms(0, 0, 0);
    let symbols: Vec<String> = std::fs::read_to_string("sp500.txt")
        .unwrap()
        .split(',')
        .map(String::from)
        .collect();
    let to = Utc::now();

    let (err_tx, mut err_rx) = tokio::sync::mpsc::channel(500);
    let (val_tx, mut val_rx) = tokio::sync::mpsc::channel(500);
    for symbol in &symbols {
        let symbol = symbol.to_string();
        // let from = from_date.clone();
        // let to = now.clone();
        let err_tx = err_tx.clone();
        let val_tx = val_tx.clone();
        let mut fetch_spec = FetchSpec { symbol, from, to };

        tokio::spawn(async move {
            match fetch_spec.execute().await {
                Ok(v) => val_tx.send(v).await.unwrap(),
                Err(e) => err_tx.send(e).await.unwrap(),
            };
        });
    }

    drop(err_tx);
    drop(val_tx);

    loop {
        tokio::select! {
            Some(e) = err_rx.recv() => {
                use std::error::Error as StdError;
                match e {
                    Error::ConnectionFailed(e) => eprintln!("{:?}", e.source()),
                    _ => {},
                    // e => eprintln!("{:?}", e),
                }
            },
            Some(_) = val_rx.recv() => {},
            // Some(v) = val_rx.recv() => println!("{:?}", v),
        }
    }
}
