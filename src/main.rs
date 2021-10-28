use chrono::prelude::*;
use yahoo::YahooConnector;
use yahoo_finance_api as yahoo;

const STOCK_SYMBOLS: [&str; 5] = ["MSFT", "GOOG", "AAPL", "UBER", "IBM"];
mod stocks;
mod util;
use stocks::*;

fn main() {
    let from = Utc.ymd(2021, 9, 1).and_hms(0, 0, 0);
    let connector = YahooConnector::new();
    for symbol in STOCK_SYMBOLS {
        match StockInfo::fetch_since(&connector, symbol, from) {
            Err(err) => eprintln!("Fetch failed: {} (\"{}\")", err, symbol),
            Ok(stock) => println!("{}", stock.fmt_csv()),
        };
    }
}
