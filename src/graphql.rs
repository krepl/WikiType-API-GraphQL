use crate::database;
use crate::database::sql::PgConnection;
use crate::database::ExerciseDao;
use crate::models;
use crate::models::{Exercise, NewExerciseBuilder};

use diesel::r2d2::{ConnectionManager, Pool};
use juniper::FieldResult;
use std::env;

// TODO: Possibly implement an error enum that can be marshaled into a GraphQL value to return as
// an error.
//
// Use https://www.restapitutorial.com/httpstatuscodes.html as a reference.
//
// TODO: Decide which module this code goes in.
impl juniper::IntoFieldError for database::Error {
    fn into_field_error(self) -> juniper::FieldError {
        use diesel::result::Error;
        match self {
            database::Error::SqlConnectionError(_) => {
                // TODO: log the connection error
                juniper::FieldError::new(
                    "Could not open connection to the database.",
                    graphql_value!({"server_error": "internal_server_error"}),
                )
            }
            database::Error::SqlError(e) => match e {
                Error::NotFound => juniper::FieldError::new(
                    "Value not found.",
                    graphql_value!({"client_error": "not_found"}),
                ),
                _ => juniper::FieldError::new(
                    e.to_string(), // TODO: Replace this direct serialization of a database error.
                    graphql_value!({"client_error": "bad_request"}),
                ),
            },
        }
    }
}

/// Simplified type for creating a new `Exercise` via the API.
///
/// This is the client-facing type which is converted into a `NewExercise` for database-insertion.
#[graphql(description = "A WikiType typing exercise.")]
#[derive(juniper::GraphQLInputObject)]
pub struct NewExercise {
    /// Title of the exercise.
    pub title: String,

    /// Content of the exercise.
    pub body: String,

    /// Optional topic describing the general exercise category.
    ///
    /// See <https://en.wikipedia.org/wiki/Portal:Contents/Portals> for an idea.
    pub topic: Option<String>,
}

impl NewExercise {
    /// Converts a `graphql::NewExercise` to a `models::NewExercise`.
    pub fn to_new_exercise_model(&self) -> models::NewExercise {
        NewExerciseBuilder::new()
            .title(&self.title)
            .body(&self.body)
            .topic(&self.topic)
            .build()
    }
}

/// TODO
pub struct Context {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Context {
    pub fn new() -> Context {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::new(database_url);
        let pool = Pool::builder().max_size(20).build(manager).unwrap();
        Context { pool }
    }
}

impl juniper::Context for Context {}

/// TODO
pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    fn apiVersion() -> &'static str {
        "1.0"
    }

    fn exercise(context: &Context, id: String) -> FieldResult<Exercise> {
        let conn: &dyn ExerciseDao = &context.pool.get().unwrap();
        let exercise = conn.find_by_id(&id)?;
        Ok(exercise)
    }
}

/// TODO
pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
    fn createExercise(context: &Context, new_exercise: NewExercise) -> FieldResult<Exercise> {
        let conn: &dyn ExerciseDao = &executor.context().pool.get().unwrap();
        let new_exercise = new_exercise.to_new_exercise_model();
        let exercise = conn.create(&new_exercise)?;
        Ok(exercise)
    }
}

/// TODO
pub type Schema = juniper::RootNode<'static, Query, Mutation>;
