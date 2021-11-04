#!/usr/bin/env bash

interval="${1:-30}"
# TODO: build only cli
cargo build --release || exit 1
# time target/release/rust-stock-tracker-cli \
#   -i "$interval" \
#   '2021-09-01T00:00:00.00Z' \
#   'AAPL,IBM,GOOG,MSFT,UBER'
time target/release/rust-stock-tracker-cli \
  -i "$interval" \
  '2021-09-01T00:00:00.00Z' \
  $(cat sp500.txt)
