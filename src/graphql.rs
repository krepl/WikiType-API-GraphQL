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
/// This is the client-facing type which is converted into a `models::NewExercise` for
/// database-insertion.
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

/// Defines shared state for GraphQL resolvers (e.g. database connections).
pub struct Context {
    // Postgres connection pool.
    //
    // NOTE: The database should already contain the `exercises` table. Otherwise, run the
    // migrations against the database.
    //
    // i.e.
    //     $ diesel migration run
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

/// Defines available non-side-effecting queries on a GraphQL endpoint.
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

/// Defines available side-effecting queries on a GraphQL endpoint.
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

/// Type alias for `juniper::RootNode<...>` (needed when implementing a GraphQL endpoint).
pub type Schema = juniper::RootNode<'static, Query, Mutation>;

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use warp::{test, Filter, Reply};

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct Exercise {
        pub id: Option<String>,
        pub title: Option<String>,
        pub body: Option<String>,
        pub topic: Option<String>,
        #[serde(rename = "createdOn")]
        pub created_on: Option<f64>,
        #[serde(rename = "modifiedOn")]
        pub modified_on: Option<f64>,
    }

    fn schema() -> Schema {
        Schema::new(Query, Mutation)
    }

    fn create_graphql_filter() -> warp::filters::BoxedFilter<(impl Reply,)> {
        let state = warp::any().map(move || Context::new());
        let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());
        let graphql_filter = warp::path("graphql").and(graphql_filter);
        graphql_filter.boxed()
    }

    fn make_test_graphql_request(request: &str) -> test::RequestBuilder {
        test::request()
            .method("POST")
            .path("/graphql")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .body(request)
    }

    /// Create the JSON-encoded body of a GraphQL POST request.
    ///
    /// Returns a `String`.
    ///
    /// # Usage
    ///
    /// ```
    /// create_graphql_request!("<query>");
    /// // or
    /// create_graphql_request!("<query>", "<variables>");
    /// // or
    /// create_graphql_request!("<query>", "<operation-name>", "<variables>");
    /// ```
    ///
    /// See [GraphQL POST request](https://graphql.org/learn/serving-over-http/#post-request).
    macro_rules! create_graphql_request {
        ($query:expr) => {
            format!(
                "{{ \
                 \"query\": \"{}\" \
                 }}",
                $query.to_string().replace('"', "\\\"")
            )
        };
        ($query:expr, $vars:expr) => {
            format!(
                "{{ \
                 \"query\": \"{}\", \
                 \"variables\": {} \
                 }}",
                $query.to_string().replace('"', "\\\""),
                $vars
            )
        };
        ($query:expr, $op_name:expr, $vars:expr) => {
            format!(
                "{{ \
                 \"query\": \"{}\", \
                 \"operationName\": \"{}\", \
                 \"variables\": {} \
                 }}",
                $query.to_string().replace('"', "\\\""),
                $op_name,
                $vars
            )
        };
    }

    #[test]
    fn create_and_fetch_exercise() {
        dotenv().ok();
        let test_database_url =
            env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        env::set_var("DATABASE_URL", test_database_url);

        let graphql_filter = create_graphql_filter();

        ///////////////////////////////////////////////////////////////////////////////
        // Create a new exercise.
        ///////////////////////////////////////////////////////////////////////////////
        let request = create_graphql_request!(
            "mutation CreateNewExercise($newExercise: NewExercise!) {
                createExercise(newExercise: $newExercise) {
                    id,
                    title,
                    body,
                    topic,
                    createdOn,
                    modifiedOn
                }
            }"
            .replace("\n", " "),
            "{
                \"newExercise\": {
                    \"title\": \"Albatross\",
                    \"body\": \"Albatrosses, of the biological family Diomedeidae, are large \
                                seabirds related to the procellariids, storm petrels, and diving \
                                petrels in the order Procellariiformes (the tubenoses).\"
                }
            }"
            .replace("\n", " ")
        );

        let response = make_test_graphql_request(&request).reply(&graphql_filter);
        let new_exercise: serde_json::Value = serde_json::from_slice(&response.body()).unwrap();
        let new_exercise: Exercise = new_exercise
            .as_object()
            .and_then(|map| map.get("data"))
            .and_then(|map| map.get("createExercise"))
            .map(|map| map.to_string())
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .unwrap()
            .unwrap();

        assert_eq!(new_exercise.title, Some(String::from("Albatross")));
        assert_eq!(new_exercise.topic, None);
        assert_ne!(new_exercise.created_on, None);
        assert_eq!(new_exercise.created_on, new_exercise.modified_on);
        assert_eq!(
            new_exercise.body,
            Some(String::from(
                "Albatrosses, of the biological family Diomedeidae, are large seabirds \
                 related to the procellariids, storm petrels, and diving petrels in the order \
                 Procellariiformes (the tubenoses)."
            ))
        );

        ///////////////////////////////////////////////////////////////////////////////
        // Query for the new exercise.
        ///////////////////////////////////////////////////////////////////////////////
        let request = create_graphql_request!(
            "query FindExerciseById($id: String!) {
                exercise(id: $id) {
                    id
                    title
                    body
                    topic
                    createdOn
                    modifiedOn
                }
            }"
            .replace("\n", " "),
            format!("{{ \"id\": \"{}\" }}", new_exercise.id.clone().unwrap())
        );

        let response = make_test_graphql_request(&request).reply(&graphql_filter);
        let exercise: serde_json::Value = serde_json::from_slice(&response.body()).unwrap();
        let exercise: Exercise = exercise
            .as_object()
            .and_then(|map| map.get("data"))
            .and_then(|map| map.get("exercise"))
            .map(|map| map.to_string())
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .unwrap()
            .unwrap();

        assert_eq!(exercise, new_exercise);
    }
}
