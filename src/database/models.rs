use super::sql::schema::exercises;
use crate::database::Result;
use std::fmt;

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
///
/// [data access object]: https://en.wikipedia.org/wiki/Data_access_object
///
/// # Examples
///
/// ```
/// use database::models::{Exercise, ExerciseDao, NewExercise, UpdatedExercise, Uuid};
/// use diesel::prelude::*;
/// use diesel::r2d2;
/// use dotenv::dotenv;
/// use std::env;
/// use std::thread;
/// use wikitype_api_graphql::database;
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
/// let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// let dao: &dyn ExerciseDao = &PgConnection::establish(&database_url)
///     .expect(&format!("Error connecting to database {}.", database_url));
///
/// // Create a new exercise.
/// let id = Uuid::new();
/// let new_exercise = NewExercise::new(&id, "Albatross", ALBATROSS_BODY, None);
///
/// // Insert the new exercise into the database.
/// let exercise = dao
///     .create(&new_exercise)
///     .expect("Failed to create Albatross exercise.");
/// println!("{:#?}", exercise);
///
/// // Create an updated exercise.
/// let updated_exercise = UpdatedExercise {
///     id: &exercise.id,
///     title: Some("Alabatross new"),
///     body: None,
///     topic: Some("It's a topic!"),
/// };
///
/// // Update the exercise.
/// let exercise = dao
///     .update(&updated_exercise)
///     .expect("Failed to create Albatross exercise.");
/// println!("{:#?}", exercise);
///
/// // Demonstrate use of r2d2::PooledConnection<M> as an ExerciseDao.
/// let manager: r2d2::ConnectionManager<PgConnection> = r2d2::ConnectionManager::new(database_url);
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
    for<'a> Create<&'a NewExercise<'a>, Exercise>
    + for<'a> FindById<&'a str, Exercise>
    + for<'a> Update<&'a UpdatedExercise<'a>, Exercise>
    + for<'a> DeleteById<&'a str, Exercise>
{
}

/// [Version 4 UUID].
///
/// Universally unique identifiers (UUID's) are used as identifiers for portability, as they can be
/// easily used in relational databases as well as non-relational databases. They are also
/// generally more scalable than, for instance, auto-incrementing id's.
///
/// [Version 4 UUID]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_(random)
pub struct Uuid {
    // NOTE: UUID's are represented as strings for increased portability.
    // - Not all database systems support UUID's natively.
    // - Aside from the automatic UUID generation provided by some database vendors, no database
    //   functionality is forfeited with this representation.
    id: String,
}

impl Uuid {
    pub fn new() -> Uuid {
        // NOTE: A v4 UUID is the recommended version for generating random, unique id's.
        //
        // See https://stackoverflow.com/questions/20342058/which-uuid-version-to-use.
        Uuid {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Representation for exercises.
#[derive(Queryable, Debug, Eq, PartialEq, Clone)]
pub struct Exercise {
    /// UUID string.
    pub id: String,

    /// Title of the exercise.
    pub title: String,

    /// Content of the exercise.
    pub body: String,

    /// Optional topic describing the general catgory of an exercise.
    ///
    /// See <https://en.wikipedia.org/wiki/Portal:Contents/Portals> for an idea.
    pub topic: Option<String>,
    // TODO
    //
    // NOTE: You can use a created_on date and a created_on time to separate out the fields. You
    // can have separate fields for year, month, day, etc. ...
    //
    //pub created_on: ...,
    //pub modified_on: ...,
}

/// Type for creating a new `Exercise`.
#[derive(Insertable)]
#[table_name = "exercises"]
pub struct NewExercise<'a> {
    id: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub topic: Option<&'a str>,
    // TODO
    //created_on: ...,
    //modified_on: ...,
}

impl<'a> NewExercise<'a> {
    pub fn get_id(&self) -> &'a str {
        &self.id
    }
}

impl<'a> NewExercise<'a> {
    pub fn new(
        id: &'a Uuid,
        title: &'a str,
        body: &'a str,
        topic: Option<&'a str>,
    ) -> NewExercise<'a> {
        let id = &id.id;
        NewExercise {
            id,
            title,
            body,
            topic,
        }
    }
}

/// Type for updating an `Exercise`.
#[derive(AsChangeset, Identifiable)]
#[table_name = "exercises"]
pub struct UpdatedExercise<'a> {
    pub id: &'a str,
    pub title: Option<&'a str>,
    pub body: Option<&'a str>,
    pub topic: Option<&'a str>,
    // TODO
    //created_on: ...,
    //modified_on: ...,
}
