#[macro_use]
extern crate diesel;

#[macro_use]
extern crate juniper;

#[cfg(test)]
#[macro_use]
extern crate serde;

#[macro_use]
extern crate serde_derive;

/// A basic [data access layer] for WikiType, including [data access objects] for a handful of SQL
/// and NoSQL databases.
///
/// [data access layer]: https://en.wikipedia.org/wiki/Data_access_layer
/// [data access objects]: https://en.wikipedia.org/wiki/Data_access_object
pub mod database;

/// GraphQL types and resolvers.
pub mod graphql;

/// Database-agnostic models for WikiType data.
pub mod models;

/// OpenID Connect 1.0 ID Token deserialization and validation.
pub mod openid_connect;
