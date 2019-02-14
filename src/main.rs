use futures::{
    self,
    future::{self, lazy, poll_fn},
    Async, Future, Poll,
};
use sql::prelude::*;
use std::{
    io::{Error, ErrorKind},
    sync::Arc,
    thread,
    time::Duration,
};
use tokio::runtime::Runtime;
use tokio_threadpool::blocking;
use tower_service::Service;

// A Connection to query from a very slow database.
struct Connection;

// DO NOT MODIFY THIS
impl Connection {
    pub fn new() -> Connection {
        Connection {}
    }

    // Our queries take about five seconds, our clients say.
    pub fn query(&self, q: &str) -> Result<(), Error> {
        thread::sleep(Duration::new(5, 0));
        Ok(println!("{}", q))
    }
}

// A very simple connection pool
struct Pool {
    connections: Vec<Connection>,
}

// DO NOT MODIFY THIS
impl Pool {
    fn new() -> Pool {
        Pool {
            connections: vec![Connection::new()],
        }
    }

    // Get a connection, if available
    fn get(&self) -> Result<&Connection, Error> {
        self.connections
            .first()
            .ok_or(Error::new(ErrorKind::Other, "OH NO!"))
    }
}

// Database handling code in Prisma, containing pool and giving us a query
// method.
struct Database {
    pool: Pool,
}

impl Database {
    pub fn new() -> Database {
        Database { pool: Pool::new() }
    }

    pub fn query(&self, query: Arc<String>) -> Result<&'static str, Error> {
        let conn = self.pool.get()?;
        conn.query(&query)?;

        Ok("DONE QUERY")
    }
}

struct DataResolver {
    database: Arc<Database>,
}

impl DataResolver {
    pub fn new() -> DataResolver {
        let database = Arc::new(Database::new());

        DataResolver { database }
    }
}

// The basic building block of the Tower library. Think of it like Twitter's
// Finagle for Scala. You define the request (SelectQuery) and response
// (empty). Future needs to be boxed to have dynamic dispatch.
impl Service<SelectQuery> for DataResolver {
    type Response = &'static str;
    type Error = Error;

    // Our future type must implement Send, so when using Tokio's threadpool
    // runtime, the futures can be sent between the threads. With the current
    // thread runtime with only one reactor core, this is not required.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error> + Send>;

    // Function to check is the connection ready to accept requests, here we
    // acceept immediately.
    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    // Call the service, returning a future. Our first dumb implementation just
    // does a blocking query and returns a simple `ok()` future when done. This
    // needs to be improved.
    fn call(&mut self, req: SelectQuery) -> Self::Future {
        let database = self.database.clone();

        let query = Arc::new(
            select_from("table")
                .columns(&["foo", "bar"])
                .so_that(req.field.equals(req.value))
                .compile()
                .unwrap(),
        );

        Box::new(lazy(move || {
            poll_fn(move || {
                let query = query.clone();
                blocking(|| database.query(query).unwrap())
                    .map_err(|_| Error::new(ErrorKind::Other, "Blocking error"))
            })
        }))
    }
}

// Query the database where field equals value
#[derive(Clone)]
struct SelectQuery {
    field: String,
    value: String,
}

fn main() {
    let mut resolver = DataResolver::new();

    let query = SelectQuery {
        field: String::from("foo"),
        value: String::from("bar"),
    };

    // The threadpool that actually runs our futures.
    let mut rt = Runtime::new().unwrap();

    // Create our two database calls, the dumb implementation will do a
    // blocking query and this actually does not help to make our system
    // faster.
    dbg!("Creating the first future");
    rt.spawn(resolver.call(query.clone()).then(|_| future::ok(())));

    dbg!("Creating the second future");
    rt.spawn(resolver.call(query.clone()).then(|_| future::ok(())));

    dbg!("Waiting for futures to finish...");
    rt.shutdown_on_idle().wait().unwrap();
}
