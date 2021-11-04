use actix::prelude::*;
use chrono::prelude::*;
use clap::clap_app;
use tokio::time;

extern crate rust_stock_tracker_lib;
use rust_stock_tracker_lib::*;

macro_rules! exit {
    ($code:expr, $template:tt, $($tt:tt)*) => {{
        eprintln!($template, $($tt)*);
        std::process::exit($code);
    }};
}

struct Args {
    from: DateTime<Utc>,
    symbols: Vec<String>,
    interval: time::Duration,
    debounce: time::Duration,
}

fn init() -> Args {
    let matches = clap_app!(my_app =>
        (version: "0.1.0")
        (author: "Till Friesewinkel [till.friesewinkel@gmail.com]")
        (about: "Fetches stock prices from the Yahoo API")
        (@arg from: +required "Starting date in %Y-%m-%d format")
        (@arg symbols: +required "Ticker symbols for the stocks to fetch")
        (@arg interval: -i --interval +takes_value "Interval between fetches in seconds (default: 30 seconds")
        (@arg debounce: -d --debounce +takes_value "Minimum delay between initializing two requests (default: 15 ms)")
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

    let interval = time::Duration::from_secs(
        matches
            .value_of("interval")
            .map(|s| match s.parse() {
                Ok(i) => i,
                Err(e) => exit!(1, "Failed to parse interval: {}", e),
            })
            .unwrap_or(30),
    );

    let debounce = time::Duration::from_millis(
        matches
            .value_of("debounce")
            .map(|s| match s.parse() {
                Ok(i) => i,
                Err(e) => exit!(1, "Failed to parse debounce: {}", e),
            })
            .unwrap_or(15),
    );

    Args {
        from,
        symbols,
        interval,
        debounce,
    }
}

#[actix_rt::main]
async fn main() {
    let args = init();
    let bufsize = args.symbols.len();

    let (ticker, tick_rx) = Ticker::new(args.interval, 5);
    let ticker = ticker.start();

    let (fetcher, fetch_rx, mut fetch_err_rx) =
        Fetcher::new(args.symbols, args.from, args.debounce);
    let fetcher = fetcher.start();
    subscribe(fetcher, tick_rx);

    let (transformer, info_rx) = Transformer::new(bufsize).unwrap();
    let transformer = transformer.start();
    subscribe(transformer, fetch_rx);

    let printer = Printer.start();
    subscribe(printer, info_rx);

    let _ = ticker.send(StartTicking).await;

    loop {
        tokio::select! {
            Some(err) = fetch_err_rx.recv() => eprintln!("{}", err)
        }
    }
}
