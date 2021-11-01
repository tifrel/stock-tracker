use actix::prelude::*;
use tokio::{io, sync::mpsc};

use crate::messages::*;

pub struct Transformer {
    info_tx: mpsc::Sender<StockInfo>,
}

impl Transformer {
    pub fn new(bufsize: usize) -> Result<(Self, mpsc::Receiver<StockInfo>), io::Error> {
        let (info_tx, info_rx) = mpsc::channel(bufsize);
        Ok((Self { info_tx }, info_rx))
    }
}

impl Actor for Transformer {
    type Context = Context<Self>;
}

impl Handler<StockHistory> for Transformer {
    type Result = ();
    fn handle(&mut self, history: StockHistory, _: &mut Context<Self>) {
        let l = history.quotes.len();
        let open = history.quotes[0].open;
        let mut high = open;
        let mut low = open;
        for q in history.quotes.iter() {
            if q.high > high {
                high = q.high;
            }
            if q.low < low {
                low = q.low;
            }
        }

        let sma30 = if l < 30 {
            None
        } else {
            Some(
                history.quotes[l - 30..]
                    .iter()
                    .map(|q| q.adjclose)
                    .fold(0.0, |sum, close| sum + close)
                    / 30.0,
            )
        };

        let info = StockInfo {
            symbol: history.symbol,
            from: history.from,
            open,
            high,
            low,
            close: (history.quotes[l - 1].adjclose / open - 1.0) * 100.0,
            sma30,
        };

        let tx = self.info_tx.clone();
        actix::spawn(async move {
            tx.send(info).await.unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! ohlcv {
        (o $o:expr, h $h:expr, l $l:expr, c $c:expr, v $v:expr) => {
            Quote {
                timestamp: 0,
                open: $o,
                high: $h,
                low: $l,
                volume: $v,
                close: $c,
                adjclose: $c,
            }
        };
    }

    #[actix_rt::test]
    async fn transform() {
        use chrono::prelude::*;
        use yahoo_finance_api::Quote;
        let (transformer, mut rx) = Transformer::new(1).unwrap();
        let transformer = transformer.start();
        let from = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);

        // basic functionality
        transformer
            .send(StockHistory {
                symbol: "AAPL".to_string(),
                from,
                quotes: vec![
                    ohlcv!(o 1.0, h 3.5, l 1.0, c 2.0, v 10),
                    ohlcv!(o 2.0, h 3.1, l 0.9, c 3.0, v 10),
                    ohlcv!(o 3.0, h 3.2, l 2.2, c 3.1, v 10),
                ],
            })
            .await
            .unwrap();
        assert_eq!(
            rx.recv().await.unwrap().fmt_csv(),
            "2021-01-01T00:00:00+00:00,AAPL,1.00,210.00,0.90,3.50,"
        );

        // no SMA until at least 30 values
        transformer
            .send(StockHistory {
                symbol: "AAPL".to_string(),
                from,
                quotes: vec![ohlcv!(o 1.0, h 1.0, l 1.0, c 1.0, v 00); 29],
            })
            .await
            .unwrap();
        assert_eq!(
            rx.recv().await.unwrap().fmt_csv(),
            "2021-01-01T00:00:00+00:00,AAPL,1.00,0.00,1.00,1.00,"
        );

        // has SMA once we have 30 values
        transformer
            .send(StockHistory {
                symbol: "AAPL".to_string(),
                from,
                quotes: vec![ohlcv!(o 1.0, h 1.0, l 1.0, c 1.0, v 00); 30],
            })
            .await
            .unwrap();
        assert_eq!(
            rx.recv().await.unwrap().fmt_csv(),
            "2021-01-01T00:00:00+00:00,AAPL,1.00,0.00,1.00,1.00,1.00"
        );
    }
}
