## What is this?

A stock tracking app, that fetches stock prices every 30 seconds (default) from
the Yahoo Finance API. The testing is done with S&P500 index, thus requiring
high throughput which is achieved by using asynchronous, non-blocking code.

## Why?

To get Rust practice and learn about async Rust :)

## Considerations and Learnings

- Simply slapping an `async/.await` everywhere does not cut it -> takes more
  than 7 minutes for S&P500
  - Once I start using `tokio::spawn`, this condenses to ca. 30 seconds
  - try resolving the Err/Ok variant inside the spawned task, not in the main
    function -> use `tokio::select` for that
  - However it is not strictly less than 30 seconds, just about averaging around
    that. Seeing the `flamegraph.svg`, more than 50% of total runtime is spent
    doing SSL handshakes. For further optimization, it might be worth it to dive
    into the corresponding functions. They might be implemented in sync,
    blocking Rust, and you might thus gain improvements by writing unblocking
    implementations. On the other hand it is security-relevant code, so
    re-implementing it should be done with caution.
  - Trying at the university reliably yields between 3.0 and 3.5 seconds. Yup,
    it's the network.
- While I like the idea of actors, Actix comes with a lot of boilerplate and
  design decisions that don't really fit the idea of this app. It also doesn't
  fit the way I like to think about applications.
  - My major pain point here is that Actix views an actor as a consumer of
    single messages. My `Ticker` however is a producer of messages, and Actix
    required me to start the production by sending a single message, whereas I
    would want that to be handled by the `Actor::start` method. This gives you
    the potential footgun of sending `StartTicking` twice. Producing messages
    required me to put a channel into the actor structs, where it would have
    been much more intuitive if those where provided by the context. The second
    part of this is the fact that we define an actors behavior by implementing a
    `handle` method that is executed for each incoming single message. Thus the
    main subject are single messages, whereas I would prefer to think in terms
    of messages streams. It lead me to defining the `subscribe` function, which
    in turn required me to return receivers from the Actor constructor. The more
    natural API would be an `actor_handle.subscribe(other)` method.
  - The minor pain point I found with Actix was the ceremony involved. This goes
    from having to define the `Actor` and `Handler` traits separately, and
    missing a `#[derive(Actor)]`, as these implementations are quite repetitive.
    While the derive for `Message` is implemented, something like
    `#[actix::message(<return type>)]` feels more appropriate.
  - There a few tests in `src/transform.rs`, mostly to show how it would
    basically work. Focus should be on battletesting and stability. Regarding
    stability, I didn't get any improper behavior in 20 minutes of running.
    Let's assume it would fail the next moment because the buffer (size = number
    of stocks = 500) couldn't handle any more messages. That means each fetching
    batch (40 total) contributed 12.5 messages to congestion and is 2.5% too
    load-heavy for the allocated time period.
  - The fetch time of a single batch can be measured by `src/bin/bench.rs`. This
    actually measures 5 batches to give an estimate of reliability of the
    readings.
