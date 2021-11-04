use actix::prelude::*;
use chrono::prelude::*;
// use tokio::time;

extern crate rust_stock_tracker_lib;
use rust_stock_tracker_lib::*;

mod bench {
    #[derive(Debug)]
    pub struct Elapsed {
        minutes: u128,
        seconds: u128,
        millis: u128,
    }

    impl Elapsed {
        pub fn since(when: std::time::Instant) -> Self {
            Self::from(std::time::Instant::now() - when)
        }
    }

    impl std::convert::From<std::time::Duration> for Elapsed {
        fn from(duration: std::time::Duration) -> Self {
            let duration = duration.as_millis();
            let millis = duration % 1000;
            let seconds = ((duration - millis) / 1000) % 60;
            let minutes = (duration - seconds - millis) / 60_000;
            Elapsed {
                minutes,
                seconds,
                millis,
            }
        }
    }

    impl std::fmt::Display for Elapsed {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}m{}.{}s", self.minutes, self.seconds, self.millis)
        }
    }

    pub struct Benchmark {
        last_iter_start: std::time::Instant,
        iters: Vec<Elapsed>,
    }

    impl Benchmark {
        pub fn start() -> Self {
            let last_iter_start = std::time::Instant::now();
            let iters = vec![];
            Self {
                last_iter_start,
                iters,
            }
        }
        pub fn iter_done(&mut self) {
            self.iters.push(Elapsed::since(self.last_iter_start));
            self.last_iter_start = std::time::Instant::now();
        }
    }

    impl std::fmt::Display for Benchmark {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            for e in self.iters.iter() {
                write!(f, "{}\n", e)?;
            }
            Ok(())
        }
    }
}
use bench::*;

#[actix_rt::main]
async fn main() {
    let from_date = Utc.ymd(2021, 9, 1).and_hms(0, 0, 0);
    let symbols = std::fs::read_to_string("sp500.txt")
        .unwrap()
        .split(',')
        .map(String::from)
        .collect();

    let mut bench = Benchmark::start();

    let (fetcher, fetch_rx, mut fetch_err_rx) =
        Fetcher::new(symbols, from_date, tokio::time::Duration::from_millis(15));
    let fetcher = fetcher.start();

    let (transformer, mut info_rx) = Transformer::new(500).unwrap();
    let transformer = transformer.start();
    subscribe(transformer, fetch_rx);

    for _ in 0..5 {
        let _ = fetcher.send(StartFetch).await;
        // wait for all 500 messages
        for _ in 0..500 {
            tokio::select! {
                Some(err) = fetch_err_rx.recv() => eprintln!("{}", err),
                Some(info) = info_rx.recv() => println!("{}", info.fmt_csv()),
            }
        }
        bench.iter_done();
    }

    println!("{}", bench);

    // TODO: runs fine on good internet, not so fine on bad internet
    // -> any way to find out number of messages stuck in
    //	channels/mailboxes/buffers?
}
