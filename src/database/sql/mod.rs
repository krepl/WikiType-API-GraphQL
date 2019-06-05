/// Auto-generated module created by Diesel from the schema defined by the migrations in
/// "migrations/" for the purpose of constructing and validating SQL queries at compile-time.
///
/// See <http://diesel.rs/guides/schema-in-depth/>.
pub mod schema;

/// Data access layer for PostgreSQL.
pub mod postgres {
    use super::schema;
    use crate::database;
    use database::models;

    pub struct PostgresExerciseDao {
        connection: diesel::PgConnection,
    }

    // TODO
    impl PostgresExerciseDao {
        pub fn new() -> PostgresExerciseDao {
            use diesel::{Connection, PgConnection};
            use dotenv::dotenv;
            use std::env;

            dotenv().ok();
            let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            // TODO: remove `expect`? perhaps return a result from `new`?
            let connection = PgConnection::establish(&database_url)
                .expect(&format!("Error connecting to PostgreSQL {}.", database_url));

            PostgresExerciseDao { connection }
        }
    }

    impl models::ExerciseDao for PostgresExerciseDao {
        fn get_exercise_by_id<'a>(&self, id: &'a str) -> database::Result<models::Exercise> {
            use database::Error::SqlError;
            use diesel::prelude::*;
            use schema::exercises::dsl::exercises;

            exercises.find(id).first(&self.connection).map_err(SqlError)
        }

        fn get_exercise_by_title<'a>(&self, title: &'a str) -> database::Result<models::Exercise> {
            use database::Error::SqlError;
            use diesel::prelude::*;
            use schema::exercises;

            exercises::table
                .filter(exercises::title.eq(title))
                .first(&self.connection)
                .map_err(SqlError)
        }

        fn create_exercise<'a>(
            &self,
            title: &'a str,
            body: &'a str,
            topic: Option<&'a str>,
        ) -> database::Result<models::Exercise> {
            use database::Error::SqlError;
            use diesel::prelude::*;
            use schema::exercises;

            let id = models::Uuid::new();

            let new_exercise = models::NewExercise::new(&id, title, body, topic);

            diesel::insert_into(exercises::table)
                .values(&new_exercise)
                .get_result(&self.connection)
                .map_err(SqlError)
        }
    }
}

/// Data access layer for SQLite.
pub mod sqlite {
    use super::schema;
    use crate::database;
    use database::models;
    use database::Error::{SqlConnectionError, SqlError};

    pub struct SqliteExerciseDao {
        connection: diesel::SqliteConnection,
    }

    impl SqliteExerciseDao {
        pub fn new_in_memory() -> database::Result<SqliteExerciseDao> {
            use diesel::prelude::*;
            use diesel::SqliteConnection;

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

    impl models::ExerciseDao for SqliteExerciseDao {
        fn get_exercise_by_id<'a>(&self, id: &'a str) -> database::Result<models::Exercise> {
            use diesel::prelude::*;
            use schema::exercises::dsl::exercises;

            exercises.find(id).first(&self.connection).map_err(SqlError)
        }

        fn get_exercise_by_title<'a>(&self, title: &'a str) -> database::Result<models::Exercise> {
            use diesel::prelude::*;
            use schema::exercises;

            exercises::table
                .filter(exercises::title.eq(title))
                .first(&self.connection)
                .map_err(SqlError)
        }

        fn create_exercise<'a>(
            &self,
            title: &'a str,
            body: &'a str,
            topic: Option<&'a str>,
        ) -> database::Result<models::Exercise> {
            use diesel::prelude::*;
            use schema::exercises;

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
    }
}

// TODO: update
#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{Exercise, Uuid};
    use sqlite::SqliteConnection;

    const ALBATROSS_BODY: &'static str = "Albatrosses, of the biological family Diomedeidae, are
        large seabirds related to the procellariids, storm petrels, and diving petrels in the order
        Procellariiformes (the tubenoses).";

    fn setup_common() -> (SqliteConnection, Exercise) {
        let conn = sqlite::new_in_memory();

        let exercise =
            sqlite::create_exercise(&conn, &Uuid::new(), "Albatross", ALBATROSS_BODY, None);
        (conn, exercise)
    }

    fn teardown_common() {}

    fn run_test_common<F: Fn(SqliteConnection, Exercise) -> ()>(f: F) {
        let (conn, e) = setup_common();
        f(conn, e);
        teardown_common();
    }

    #[test]
    fn it_works() {
        run_test_common(|conn, e| {
            let e2 = sqlite::get_exercise_by_title(&conn, "Albatross")
                .expect("Test exercise not found.");
            assert_eq!(e, e2);
        });
    }
}
