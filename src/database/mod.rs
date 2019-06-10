use crate::models::{Exercise, NewExercise, UpdatedExercise};

use std::fmt;
use std::result;

/// SQL schemas and DAO implementations.
pub mod sql;

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

/// Error type returned by databases-related functions.
///
/// TODO: Elaborate on the types of errors?
#[derive(Debug, PartialEq)]
pub enum Error {
    SqlError(diesel::result::Error),
    SqlConnectionError(diesel::result::ConnectionError),
    // TODO
    //NoSqlError(),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Result type returned by databases-related functions.
pub type Result<T> = result::Result<T, Error>;

/// Generic create operation.
pub trait Create<T, R> {
    fn create(&self, obj: T) -> Result<R>;
}

/// Generic find-by-id operation.
pub trait FindById<ID, R> {
    fn find_by_id(&self, id: ID) -> Result<R>;
}

/// Generic update operation.
pub trait Update<T, R> {
    fn update(&self, obj: T) -> Result<R>;
}

/// Generic delete operation.
pub trait DeleteById<ID, R> {
    fn delete_by_id(&self, id: ID) -> Result<R>;
}

/// A [data access object] for exercises.
///
/// Current implementors include
/// - `diesel::PgConnection`
/// - `diesel::MysqlConnection`
/// - `diesel::r2d2::PooledConnection`
/// - `wikitype_api::database::sql::SqliteConnection`
///
/// [data access object]: https://en.wikipedia.org/wiki/Data_access_object
///
/// # Examples
///
/// ```
/// use database::ExerciseDao;
/// use diesel::prelude::*;
/// use diesel::r2d2;
/// use dotenv::dotenv;
/// use std::env;
/// use std::thread;
/// use wikitype_api::database;
/// use wikitype_api::models::{Exercise, NewExerciseBuilder, UpdatedExerciseBuilder};
///
/// const ALBATROSS_BODY: &'static str =
///     "Albatrosses, of the biological family Diomedeidae, are large seabirds related to the \
///      procellariids, storm petrels, and diving petrels in the order Procellariiformes (the \
///      tubenoses).";
///
/// // Connect to a Postgres database.
/// //
/// // NOTE: This database should already contain the `exercises` table. Otherwise, run the
/// // migrations against the database.
/// //
/// // i.e.
/// //     $ diesel migration run
/// dotenv().ok();
///
/// let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
/// let dao: &dyn ExerciseDao = &PgConnection::establish(&database_url)
///     .expect(&format!("Error connecting to database {}.", database_url));
///
/// // Create a new exercise.
/// let new_exercise = NewExerciseBuilder::new()
///     .title("Albatross")
///     .body(ALBATROSS_BODY)
///     .build();
///
/// // Insert the new exercise into the database.
/// let exercise = dao
///     .create(&new_exercise)
///     .expect("Failed to create Albatross exercise.");
/// assert_eq!(&exercise.id, new_exercise.get_id());
/// assert_eq!(exercise.title, new_exercise.title);
/// assert_eq!(exercise.body, new_exercise.body);
/// assert_eq!(exercise.topic.is_none(), new_exercise.topic.is_none());
///
/// // Create an updated exercise.
/// let updated_exercise = UpdatedExerciseBuilder::new(&exercise)
///     .title("Albatross new")
///     .topic(Some("It's a topic!"))
///     .build();
///
/// // Update the exercise.
/// let exercise = dao
///     .update(&updated_exercise)
///     .expect("Failed to create Albatross exercise.");
/// assert_eq!(updated_exercise.get_id(), &exercise.id);
/// assert_eq!(&exercise.title, "Albatross new");
/// assert_eq!(&exercise.body, ALBATROSS_BODY);
/// assert_eq!(exercise.topic, Some(String::from("It's a topic!")));
///
/// // Demonstrate use of r2d2::PooledConnection<M> as an ExerciseDao.
/// let manager: r2d2::ConnectionManager<PgConnection> =
///     r2d2::ConnectionManager::new(database_url);
/// let pool = r2d2::Pool::builder().max_size(10).build(manager).unwrap();
///
/// let join_handles: Vec<thread::JoinHandle<database::Result<Exercise>>> = (0..20)
///     .map(|_| {
///         let pool = pool.clone();
///         let exercise = exercise.clone();
///         thread::spawn(move || {
///             let conn: &dyn ExerciseDao = &pool.get().unwrap();
///             conn.find_by_id(&exercise.id)
///         })
///     })
///     .collect();
///
/// for jh in join_handles {
///     assert_eq!(jh.join().unwrap(), Ok(exercise.clone()));
/// }
///
/// // Delete the exercise.
/// let deleted_exercise = dao
///     .delete_by_id(&exercise.id)
///     .expect("Failed to create Albatross exercise.");
/// assert_eq!(exercise, deleted_exercise);
///
/// let exercise = dao.find_by_id(&exercise.id);
/// assert_eq!(
///     exercise,
///     Err(database::Error::SqlError(diesel::result::Error::NotFound))
/// );
/// ```
pub trait ExerciseDao:
    for<'a> Create<&'a NewExercise, Exercise>
    + for<'a> FindById<&'a str, Exercise>
    + for<'a> Update<&'a UpdatedExercise<'a>, Exercise>
    + for<'a> DeleteById<&'a str, Exercise>
{
}
