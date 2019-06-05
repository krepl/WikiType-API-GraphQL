//! A basic [data access layer] for WikiType, including models for WikiType data types and [data
//! access objects] for a handful of SQL and NoSQL databases.
//!
//! [data access layer]: https://en.wikipedia.org/wiki/Data_access_layer
//! [data access objects]: https://en.wikipedia.org/wiki/Data_access_object

/// Database-agnostic models for WikiType data.
pub mod models;

/// SQL data access layer.
pub mod sql;

use std::result;

/// TODO
#[derive(Debug)]
pub enum Error {
    SqlError(diesel::result::Error),
    SqlConnectionError(diesel::result::ConnectionError),
    // TODO
    //NoSqlError(),
}

type Result<T> = result::Result<T, Error>;

///// TODO
//pub mod nosql {
//pub mod document {
//pub mod mongodb {}

//// ... e.g. firebase, couchbase, amazon documentdb
//}

///// Redis is a distributed, in-memory key-value database with optional durability.
/////
///// It can be useful for caching and offers persistence if needed. Typically Redis is
///// configured to not offer durability, i.e. recent, committed transactions may be lost, which
///// helps with performance.
//pub mod redis {}
//}
