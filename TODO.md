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
- [x] Code polishing
  - [x] Testing for function limitations
  - [x] Do I miss data because of backlog? -> Depends on the internet
        connection, see considerations in `README.md`
  - [x] Other ways to implement this pipeline? Of course there are multiple
        ways. I am beginning to like actors more and more, but my view on
        `actix` can be found in `README.md`. My main consideration in terms of
        architecture would be splitting the `Fetch` actor, such that it doesn't
        require me to spawn tasks inside the `handle` function, i.e. a
        `FetchTrigger` actor that, triggered by each `StartFetch` message, sends
        500 `Fetch(<symbol>, <from>)` to a `FetchSingle` actor.
- [x] Test with all the S&P500 symbols

### Milestone 3

### Milestone 4
