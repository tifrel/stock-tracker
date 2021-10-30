#!/usr/bin/env bash

cargo build --release
sudo cargo flamegraph -- '2021-09-01T00:00:00.00Z' $(cat sp500.txt)
