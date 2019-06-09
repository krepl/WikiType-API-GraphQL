use crate::database;
use database::models::{
    Create, DeleteById, Exercise, ExerciseDao, FindById, NewExercise, Update, UpdatedExercise,
};
use database::Error::SqlError;
use database::Result;
use diesel::backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax};
use diesel::prelude::*;
use schema::*;

pub use diesel::mysql::MysqlConnection;
pub use diesel::pg::PgConnection;
pub use diesel::r2d2::ConnectionManager;
pub use diesel::r2d2::PooledConnection;

/// Auto-generated module created by Diesel from the schema defined by the migrations in
/// "migrations/" for the purpose of constructing and validating SQL queries at compile-time.
///
/// See <http://diesel.rs/guides/schema-in-depth/>.
pub mod schema;

// NOTE: Generic type DB must have an explicit lifetime to ensure that any values that contain
// references to <Conn as Connection>::Backend do not outlive any references in the DB type itself.
//
// To the best of my knowledge, all diesel types that implement diesel::backend::Backend do not
// contain references and, to the borrow checker, are indistinguishable from types whose references
// all have static lifetimes. Thus, the "DB: 'static" lifetime bound is trivially satisfied by all
// diesel Backend implementations.
//
// See https://doc.rust-lang.org/book/ch19-02-advanced-lifetimes.html#lifetime-bounds-on-references-to-generic-types.

/// Blanket `ExerciseDao` implementation for SQL backends.
impl<'a, Conn, DB: 'static> ExerciseDao for Conn
where
    Conn: for<'b> FindById<&'b str, Exercise>,
    Conn: Connection<Backend = DB>,
    DB: Backend<RawValue = [u8]>,
    DB: SupportsDefaultKeyword,
    DB: UsesAnsiSavepointSyntax,
{
}

impl<'a, Conn, DB: 'static> Create<&'a NewExercise<'a>, Exercise> for Conn
where
    Conn: for<'b> FindById<&'b str, Exercise>,
    Conn: Connection<Backend = DB>,
    DB: Backend,
    DB: SupportsDefaultKeyword,
{
    fn create(&self, obj: &NewExercise) -> Result<Exercise> {
        diesel::insert_into(exercises::table)
            .values(obj)
            .execute(self)
            .map_err(SqlError)?;

        self.find_by_id(obj.get_id())
    }
}

impl<'a, Conn, DB: 'static> FindById<&'a str, Exercise> for Conn
where
    Conn: Connection<Backend = DB>,
    DB: Backend<RawValue = [u8]>,
    DB: UsesAnsiSavepointSyntax,
{
    fn find_by_id(&self, id: &'a str) -> Result<Exercise> {
        exercises::table.find(id).first(self).map_err(SqlError)
    }
}

impl<'a, Conn, DB: 'static> Update<&'a UpdatedExercise<'a>, Exercise> for Conn
where
    Conn: for<'b> FindById<&'b str, Exercise>,
    Conn: Connection<Backend = DB>,
    DB: Backend,
    DB: SupportsDefaultKeyword,
{
    fn update(&self, obj: &'a UpdatedExercise<'a>) -> Result<Exercise> {
        diesel::update(exercises::table)
            .set(obj)
            .execute(self)
            .map_err(SqlError)
            .and_then(|_| self.find_by_id(&obj.get_id()))
    }
}

impl<'a, Conn, DB: 'static> DeleteById<&'a str, Exercise> for Conn
where
    Conn: Connection<Backend = DB>,
    Conn: for<'b> FindById<&'b str, Exercise>,
    DB: Backend,
{
    fn delete_by_id(&self, id: &'a str) -> Result<Exercise> {
        let exercise = self.find_by_id(id);
        diesel::delete(exercises::table.find(id))
            .execute(self)
            .map_err(SqlError)
            .and_then(|_| exercise)
    }
}

/// Newtype for implementing `ExerciseDao` on a `diesel::sqlite::SqliteConnection` without
/// conflicting with the blanket `ExerciseDao` implementation for SQL backends.
///
/// # Examples
///
/// ```
/// use database::models::{Exercise, ExerciseDao, NewExercise, Uuid};
/// use diesel::prelude::*;
/// use wikitype_api_graphql::database;
///
/// const ALBATROSS_BODY: &'static str =
///     "Albatrosses, of the biological family Diomedeidae, are large seabirds related to the \
///      procellariids, storm petrels, and diving petrels in the order Procellariiformes (the \
///      tubenoses).";
///
/// // Create an in-memory SQLite database and create the `exercises` table.
/// let dao = database::sql::SqliteConnection(
///     SqliteConnection::establish(":memory:")
///         .expect(&format!("Error creating in-memory SQLite database.")),
/// );
/// let create_table =
///     std::fs::read_to_string("./migrations/2019-06-02-153217_create_exercises/up.sql").unwrap();
/// diesel::sql_query(create_table).execute(&dao.0).unwrap();
///
/// let dao: &dyn ExerciseDao = &dao;
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
pub struct SqliteConnection(pub diesel::SqliteConnection);

impl ExerciseDao for SqliteConnection {}

impl<'a> Create<&'a NewExercise<'a>, Exercise> for SqliteConnection {
    fn create(&self, obj: &NewExercise) -> Result<Exercise> {
        diesel::insert_into(exercises::table)
            .values(obj)
            .execute(&self.0)
            .map_err(SqlError)?;

        self.find_by_id(obj.get_id())
    }
}

impl<'a> FindById<&'a str, Exercise> for SqliteConnection {
    fn find_by_id(&self, id: &'a str) -> Result<Exercise> {
        exercises::table.find(id).first(&self.0).map_err(SqlError)
    }
}

impl<'a> Update<&'a UpdatedExercise<'a>, Exercise> for SqliteConnection {
    fn update(&self, obj: &'a UpdatedExercise<'a>) -> Result<Exercise> {
        diesel::update(exercises::table)
            .set(obj)
            .execute(&self.0)
            .map_err(SqlError)
            .and_then(|_| self.find_by_id(&obj.get_id()))
    }
}

impl<'a> DeleteById<&'a str, Exercise> for SqliteConnection {
    fn delete_by_id(&self, id: &'a str) -> Result<Exercise> {
        let exercise = self.find_by_id(id);
        diesel::delete(exercises::table.find(id))
            .execute(&self.0)
            .map_err(SqlError)
            .and_then(|_| exercise)
    }
}
