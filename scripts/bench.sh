#!/usr/bin/env bash

# TODO: build only cli
cargo build --release || exit 1
time target/release/rust-stock-tracker-bench
