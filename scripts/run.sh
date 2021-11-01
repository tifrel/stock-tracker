#!/usr/bin/env bash

# cargo run -- '2021-09-01T00:00:00.00Z' 'AAPL,IBM,GOOG,MSFT,UBER'

cargo build --release || exit 1
# time target/release/rust-stock-tracker '2021-09-01T00:00:00.00Z' 'AAPL,IBM,GOOG,MSFT,UBER'
time target/release/rust-stock-tracker '2021-09-01T00:00:00.00Z' $(cat sp500.txt)
