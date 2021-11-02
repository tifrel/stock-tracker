## Why?

To get Rust practice and learn about async Rust :)

## Deliverables

### Milestone one

- [x] Use `yahoo_finance_api` to fetch prices. For now, we stick with the
      `"blocking"` feature before we move to async later on
- [x] CLI arguments: stock symbols, from-date
  - [x] use clap for this
- [x] indicators
  - [x] `n_window_sma`
  - [x] `price_diff`
- [x] Print a csv to `stdout`, and errors to `stderr`
  - [x] CSV columns: Date of last quote, stock symbol, close price for last
        quote, percentage change since open, high, low, 30d-MA
  - [x] Numbers get a max of two decimal places

### Milestone 2

- [x] Write tests -> Already did that before, because TDD
- [x] Transform the code to be async
  - [x] I will go for [`tokio`](https://tokio.rs/), as I feel it to be the most
        used. Stats on crates.io confirm that.
- [x] Use actors for the data processing
  - [x] Wrapping code to work in actors
  - [x] Publish and subscribe to messages without explicit calls
  - [x] For `tokio`, one should use [`actix`](https://crates.io/crates/actix)
- [x] Continuous fetching of stock quotes for each symbols
  - [x] needs to happen every 30 seconds
  - [x] Never sleep the thread -> Actors
  - [x] Actors for the processing
- [ ] Code polishing
  - [x] Testing for function limitations
  - [x] Do I miss data because of backlog?
  - [ ] Other ways to implement this pipeline?
- [x] Test with all the S&P500 symbols

### Milestone 3

### Milestone 4

## Take-home messages

- Simply slapping an `async/.await` everywhere does not cut it -> takes more
  than 7 minutes for S&P500
  - Once I start using `tokio::spawn`, this condenses to ca. 30 seconds
  - try resolving the Err/Ok variant inside the spawned task, not in the main
    function -> use `tokio::select` for that
  - However it is not strictly less than 30 seconds, just about averaging around
    that. Seeing the `flamegraph.svg`, more than 50% of total runtime is spent
    doing SSL handshakes.
  - Trying at the university reliably yields between 3.0 and 3.5 seconds. Yup,
    it's the network.
- Actix seems like a lot of boilerplate, I wouldn't necessarily use it, but it's
  part of the requirements...
  - There a few tests in `src/transform.rs`, mostly to show how it would
    basically work. Focus should be on battletesting and stability. Regarding
    stability, I didn't get any improper behavior in 20 minutes of running.
    Let's assume it would fail the next moment because the buffer (size = number
    of stocks = 500) couldn't handle any more messages. That means each fetching
    batch (40 total) contributed 12.5 messages to congestion and is 2.5% too
    load-heavy for the allocated time period.
  - One problem with the Actor-based system is the increased complexity for
    measuring the fetch time of a single batch, but might be handled in a
    specific binary compiled for that purpose. (Done in `src/bin/bench.rs`)
