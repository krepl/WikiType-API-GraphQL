#[macro_use]
extern crate diesel;

/// A basic [data access layer] for WikiType, including [data access objects] for a handful of SQL
/// and NoSQL databases.
///
/// [data access layer]: https://en.wikipedia.org/wiki/Data_access_layer
/// [data access objects]: https://en.wikipedia.org/wiki/Data_access_object
pub mod database;

/// TODO
pub mod graphql;

/// Database-agnostic models for WikiType data.
pub mod models;
