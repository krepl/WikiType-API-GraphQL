// TODO: Make use of diesel::r2d2 support for connection pooling.

use crate::database;
use database::models;
use database::Error::{SqlConnectionError, SqlError};
use diesel::prelude::*;
use schema::*;

/// Auto-generated module created by Diesel from the schema defined by the migrations in
/// "migrations/" for the purpose of constructing and validating SQL queries at compile-time.
///
/// See <http://diesel.rs/guides/schema-in-depth/>.
pub mod schema;

// Internal macro for code-reuse in ExerciseDao implementations.
macro_rules! sql_exercise_dao_create_exercise {
    ($struct_name:ident, PgConnection) => {
        fn create_exercise<'a>(
            &self,
            title: &'a str,
            body: &'a str,
            topic: Option<&'a str>,
        ) -> database::Result<models::Exercise> {
            let id = models::Uuid::new();
            let new_exercise = models::NewExercise::new(&id, title, body, topic);
            diesel::insert_into(exercises::table)
                .values(&new_exercise)
                .get_result(&self.connection)
                .map_err(SqlError)
        }
    };
    ($struct_name:ident, MysqlConnection) => {
        sql_exercise_dao_create_exercise!($struct_name, SqliteConnection);
    };
    ($struct_name:ident, SqliteConnection) => {
        fn create_exercise<'a>(
            &self,
            title: &'a str,
            body: &'a str,
            topic: Option<&'a str>,
        ) -> database::Result<models::Exercise> {
            let id = models::Uuid::new();
            let new_exercise = models::NewExercise::new(&id, title, body, topic);
            diesel::insert_into(exercises::table)
                .values(&new_exercise)
                .execute(&self.connection)
                .map_err(SqlError)?;

            exercises::table
                .filter(exercises::id.eq(new_exercise.id))
                .first::<models::Exercise>(&self.connection)
                .map_err(SqlError)
        }
    };
}

// Internal macro for code-reuse in ExerciseDao implementations.
macro_rules! sql_exercise_dao {
    ($(#[$attr:meta])* $struct_name:ident, $connection_type:ident) => {
        $(#[$attr])*
        pub struct $struct_name {
            connection: $connection_type,
        }

        impl models::ExerciseDao for $struct_name {
            sql_exercise_dao_create_exercise!($struct_name, $connection_type);

            fn get_exercise_by_id<'a>(&self, id: &'a str) -> database::Result<models::Exercise> {
                exercises::table
                    .find(id)
                    .first(&self.connection)
                    .map_err(SqlError)
            }

            fn get_exercise_by_title<'a>(
                &self,
                title: &'a str,
            ) -> database::Result<models::Exercise> {
                exercises::table
                    .filter(exercises::title.eq(title))
                    .first(&self.connection)
                    .map_err(SqlError)
            }
        }
    };
}

// Internal macro for code-reuse in ExerciseDao implementations.
macro_rules! sql_exercise_dao_new {
    ($struct_name:ident, $connection_type:ident) => {
        impl $struct_name {
            pub fn new() -> $struct_name {
                use dotenv::dotenv;
                use std::env;

                dotenv().ok();
                let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
                // TODO: remove `expect`? perhaps return a `Result` from `new`?
                let connection = $connection_type::establish(&database_url)
                    .expect(&format!("Error connecting to database {}.", database_url));

                $struct_name { connection }
            }
        }
    };
}

sql_exercise_dao!(
    /// Data access object for PostgreSQL.
    PostgresExerciseDao,
    PgConnection
);
sql_exercise_dao_new!(PostgresExerciseDao, PgConnection);

sql_exercise_dao!(
    /// Data access object for MySQL.
    MysqlExerciseDao,
    MysqlConnection
);
sql_exercise_dao_new!(MysqlExerciseDao, MysqlConnection);

sql_exercise_dao!(
    /// Data access object for SQLite.
    SqliteExerciseDao,
    SqliteConnection
);

impl SqliteExerciseDao {
    /// Creates a new, in-memory SQLite database and returns a `SqliteExerciseDao` for it.
    ///
    /// Useful for testing.
    ///
    /// # Examples
    /// ```
    /// use wikitype_api_graphql::database::models::{Exercise, ExerciseDao};
    /// use wikitype_api_graphql::database::sql::SqliteExerciseDao;
    ///
    /// const ALBATROSS_BODY: &'static str = "Albatrosses, of the biological family Diomedeidae, are
    ///     large seabirds related to the procellariids, storm petrels, and diving petrels in the order
    ///     Procellariiformes (the tubenoses).";
    ///
    /// let dao = SqliteExerciseDao::new_in_memory()
    ///     .expect("Failed to create in-memory SQLite database.");
    ///
    /// let exercise = dao
    ///     .create_exercise("Albatross", ALBATROSS_BODY, None)
    ///     .expect("Failed to create test exercise.");
    ///
    /// let e = dao
    ///     .get_exercise_by_title("Albatross")
    ///     .expect("Test exercise not found.");
    ///
    /// assert_eq!(exercise, e);
    /// ```
    pub fn new_in_memory() -> database::Result<SqliteExerciseDao> {
        let connection = SqliteConnection::establish(":memory:").map_err(SqlConnectionError)?;

        // TODO: replace `expect`?
        // TODO: remove hard-coded string and dependency on a single schema.
        let query =
            std::fs::read_to_string("./migrations/2019-06-02-153217_create_exercises/up.sql")
                .expect("Failed to read SQL schema.");

        diesel::sql_query(query)
            .execute(&connection)
            .map_err(SqlError)?;

        Ok(SqliteExerciseDao { connection })
    }
}

// TODO: Add tests for other backends.
#[cfg(test)]
mod tests {
    use crate::database::models::{Exercise, ExerciseDao};
    use crate::database::sql::SqliteExerciseDao;

    const ALBATROSS_BODY: &'static str = "Albatrosses, of the biological family Diomedeidae, are
        large seabirds related to the procellariids, storm petrels, and diving petrels in the order
        Procellariiformes (the tubenoses).";

    fn setup_common() -> (Box<ExerciseDao>, Exercise) {
        let dao = SqliteExerciseDao::new_in_memory()
            .expect("Failed to create in-memory SQLite database.");

        let exercise = dao
            .create_exercise("Albatross", ALBATROSS_BODY, None)
            .expect("Failed to create test exercise.");
        (Box::new(dao), exercise)
    }

    fn teardown_common() {}

    fn run_test_common<F: Fn(Box<ExerciseDao>, Exercise) -> ()>(f: F) {
        let (conn, e) = setup_common();
        f(conn, e);
        teardown_common();
    }

    #[test]
    fn it_works() {
        run_test_common(|dao, e| {
            let e2 = dao
                .get_exercise_by_title("Albatross")
                .expect("Test exercise not found.");
            assert_eq!(e, e2);
        });
    }
}
