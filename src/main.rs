use chrono::prelude::*;
use clap::clap_app;
use tokio;
use yahoo::YahooConnector;
use yahoo_finance_api as yahoo;

extern crate rust_stock_tracker_lib;
use rust_stock_tracker_lib::stocks::*;

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
        Err(e) => exit!(1, "Failed to parse start date: {}", e),
        Ok(dt) => dt.with_timezone(&Utc),
    };

    Args { from, symbols }
}

#[tokio::main]
async fn main() {
    let args = init();
    let from = args.from;

    let (tx, mut rx) = tokio::sync::mpsc::channel(args.symbols.len());

    // send of the fetching to different tasks
    let connector_arc = std::sync::Arc::new(YahooConnector::new());
    for symbol in args.symbols {
        let connector = connector_arc.clone();
        // need to clone for borrow checker
        let tx = tx.clone();
        tokio::spawn(async move {
            let info = StockInfo::fetch_since(&connector, &symbol, from).await;
            tx.send((symbol, info)).await.unwrap();
        });
    }

    drop(tx); // We need this drop for the program to terminate

    loop {
        match rx.recv().await {
            Some((symbol, Err(err))) => eprintln!("Fetch failed: {} (\"{}\")", err, symbol),
            Some((_, Ok(stock))) => println!("{}", stock.fmt_csv()),
            None => std::process::exit(0),
        }
    }
}
