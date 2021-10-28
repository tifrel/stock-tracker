use chrono::prelude::*;
use clap::clap_app;
use yahoo::YahooConnector;
use yahoo_finance_api as yahoo;

mod stocks;
mod util;
use stocks::*;

struct Args {
    from: DateTime<Utc>,
    symbols: Vec<String>,
}

macro_rules! exit {
    ($code:expr, $template:tt, $($tt:tt)*) => {{
        eprintln!($template, $($tt)*);
        std::process::exit($code);
    }};
}

fn init() -> Args {
    let matches = clap_app!(my_app =>
        (version: "0.1.0")
        (author: "Till Friesewinkel [till.friesewinkel@gmail.com]")
        (about: "Fetches stock prices from the Yahoo API")
        (@arg from: +required "Starting date in %Y-%m-%d format")
        (@arg symbols: +required "Ticker symbols for the stocks to fetch")
    )
    .get_matches();

    let symbols = matches
        .value_of("symbols")
        .unwrap()
        .split(",")
        .map(|s| s.to_string())
        .collect();

    let from = match DateTime::parse_from_rfc3339(matches.value_of("from").unwrap()) {
        Err(e) => {
            exit!(1, "Failed to parse start date: {}", e)
        }
        Ok(dt) => dt.with_timezone(&Utc),
    };

    Args { from, symbols }
}

fn main() {
    let args = init();
    let connector = YahooConnector::new();
    for symbol in args.symbols {
        match StockInfo::fetch_since(&connector, &symbol, args.from) {
            Err(err) => eprintln!("Fetch failed: {} (\"{}\")", err, symbol),
            Ok(stock) => println!("{}", stock.fmt_csv()),
        };
    }
}
