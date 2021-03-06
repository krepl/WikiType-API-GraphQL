use crate::models::{Exercise, NewExercise, UpdatedExercise};

use diesel::result::ConnectionError as DieselConnectionError;
use diesel::result::Error as DieselError;
use std::fmt;
use std::result;

/// SQL schemas and DAO implementations.
pub mod sql;

/// Error type returned by databases-related functions.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The requested resource could not be found.
    NotFound,

    /// The database query could not be constructed.
    QueryError(String),

    /// An error occurred deserializing the data being sent to the database.
    DeserializationError(String),

    /// An error occurred serializing the data being sent to the database.
    SerializationError(String),

    /// A catchall error for general server errors.
    ServerError(Option<String>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Trait for converting other error types into `database::Error`s.
pub trait IntoDatabaseError {
    fn into_database_error(self) -> Error;
}

impl IntoDatabaseError for DieselError {
    fn into_database_error(self) -> Error {
        match self {
            DieselError::NotFound => Error::NotFound,
            DieselError::QueryBuilderError(e) => Error::QueryError(e.to_string()),
            DieselError::DeserializationError(e) => Error::DeserializationError(e.to_string()),
            DieselError::SerializationError(e) => Error::SerializationError(e.to_string()),
            e => Error::ServerError(Some(e.to_string())),
        }
    }
}

impl IntoDatabaseError for DieselConnectionError {
    fn into_database_error(self) -> Error {
        match self {
            e => Error::ServerError(Some(e.to_string())),
        }
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
/// let updated_exercise = UpdatedExerciseBuilder::new(&exercise.id)
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
///     Err(database::Error::NotFound)
/// );
/// ```
pub trait ExerciseDao:
    for<'a> Create<&'a NewExercise, Exercise>
    + for<'a> FindById<&'a str, Exercise>
    + for<'a> Update<&'a UpdatedExercise<'a>, Exercise>
    + for<'a> DeleteById<&'a str, Exercise>
{
}
