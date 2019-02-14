# Rust Quiz 3

This time we take a look into concurrency and parallel execution in Rust. In
this example we have a slow database connection and we must make the queries to
execute in parallel to speed up our service.

## The sync version

The first version of the code is available by checking out the [first
commit](https://github.com/pimeys/tokio_quiz/commit/efbff2e6229a0acceae0f8233f982d0f64a44435#diff-639fbc4ef05b315af92b4d836c31b023).

The program runs two queries in sequence, each taking five seconds to load, and
being sequental, the whole program executes in ten seconds. We can do better
than that.

## Introducing Futures, Tokio and Tower

These are the common building blocks for concurrent rust. Future is [a
trait](https://tokio.rs/docs/futures/basic/) and a Future implementation should
implement the poll method, which responds with either `Ready` including the
response value, or `NotReady` and the future should be called again later to
check the status. You can call poll only until a value is returned.

Futures 0.1 is the version to use. 0.2 never went anywhere and is abandoned.
The 0.3 version works only with nightly rust.

You can always call `.wait()` from the future to execute it immediately and
block the current thread.

Tokio is a set of libraries for executing futures. The most important is
the runtime, that takes the future and calls the poll in a performant way. The
default implementation has a pool of threads running the poll function in
parallel and if the future is doing network IO, it will run in a specialized
reactor core thread able to handle millions of concurrent futures.

Tower gives building blocks to implement network services. The only stabilized
block is the `Service` trait, that defines the request, response, error and
future types, and offers two functions: `.poll_ready` to see if the service is
up and running, and `.call` to send a request and get a response future back.

The [second
version](https://github.com/pimeys/tokio_quiz/commit/dccdb98e58b24c65246c0eea7e06f6520ddabe80#diff-639fbc4ef05b315af92b4d836c31b023)
of this quiz combines the libraries with the dumbest possible way, offering no
actual concurrency by running blocking code in a concurrent block.

Please run both versions and understand the code what it does.

## The Quiz Question

Implement a version, where the `call` in service actually does not block and
returns immediately a future. You are not allowed to modify our connection, so
you need to accept your fate and wrap the blocking code so that it works fast
with the runtime, in 5 seconds instead of 10.

Tokio-threadpool offers the
[blocking](https://docs.rs/tokio-threadpool/0.1.11/tokio_threadpool/fn.blocking.html)
function, that can help with solving the problem. Read the documentation for
blocking, and for [tokio](https://tokio.rs/docs/overview/), and maybe also for
[futures](https://docs.rs/futures/0.1.25/futures/).

You should be able to get it working by modifying the Database and its Service
implementation. You should not touch the Pool or Connection.
