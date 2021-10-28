## Why?

To get Rust practice and learn about async Rust :)

### Deliverables

#### Milestone one

- Use `yahoo_finance_api` to fetch prices. For now, we stick with the
  `"blocking"` feature before we move to async later on
- CLI arguments: stock symbols, from-date
  - use clap for this
- indicators
  - `n_window_sma`
  - `price_diff`
- Print a csv to `stdout`, and errors to `stderr`
  - CSV columns: Date of last quote, stock symbol, close price for last quote,
    percentage change since open, high, low, 30d-MA
  - Numbers get a max of two decimal places

### Take-home messages
