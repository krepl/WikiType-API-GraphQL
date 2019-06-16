use crate::database;
use crate::database::sql::PgConnection;
use crate::database::ExerciseDao;
use crate::models;
use crate::models::{Exercise, NewExerciseBuilder, UpdatedExerciseBuilder};

use diesel::r2d2::{ConnectionManager, Pool};
use std::env;

/// Error-handling for database errors returned from resolvers.
///
/// Use <https://www.restapitutorial.com/httpstatuscodes.html> as a reference.
impl juniper::IntoFieldError for database::Error {
    fn into_field_error(self) -> juniper::FieldError {
        use database::Error;
        match self {
            Error::NotFound => juniper::FieldError::new(
                "Resource not found",
                graphql_value!({"client_error": "not_found"}),
            ),
            Error::QueryError(e)
            | Error::DeserializationError(e)
            | Error::SerializationError(e) => {
                juniper::FieldError::new(e, graphql_value!({"client_error": "bad_request"}))
            }
            Error::ServerError(e) => {
                if let Some(e) = e {
                    log::error!("{}", e);
                }
                juniper::FieldError::new(
                    "An internal server error occurred",
                    graphql_value!({"server_error": "internal_server_error"}),
                )
            }
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
            .topic(self.topic.as_ref().map(|t| &**t))
            .build()
    }
}

/// Simplified type for updating an `Exercise` via the API.
///
/// This is the client-facing type which is converted into a `models::UpdatedExercise` for
/// updating.
#[graphql(description = "A WikiType typing exercise.")]
#[derive(juniper::GraphQLInputObject)]
pub struct UpdatedExercise {
    /// UUID string.
    pub id: String,

    /// Title of the exercise.
    pub title: Option<String>,

    /// Content of the exercise.
    pub body: Option<String>,

    /// Optional topic describing the general exercise category.
    ///
    /// See <https://en.wikipedia.org/wiki/Portal:Contents/Portals> for an idea.
    pub topic: Option<String>,
}

impl UpdatedExercise {
    /// Converts a `graphql::UpdatedExercise` to a `models::UpdatedExercise`.
    pub fn to_updated_exercise_model(&self) -> models::UpdatedExercise {
        let mut update = UpdatedExerciseBuilder::new(&self.id);
        self.title.as_ref().map(|title| update.title(title));
        self.body.as_ref().map(|body| update.body(body));
        update.topic(self.topic.as_ref().map(|t| &**t));
        update.build()
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

    fn exercise(context: &Context, id: String) -> Result<Exercise, database::Error> {
        let conn: &dyn ExerciseDao = &context.pool.get().unwrap();
        let exercise = conn.find_by_id(&id)?;
        Ok(exercise)
    }
}

/// Defines available side-effecting queries on a GraphQL endpoint.
pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
    fn createExercise(
        context: &Context,
        new_exercise: NewExercise,
    ) -> Result<Exercise, database::Error> {
        let conn: &dyn ExerciseDao = &executor.context().pool.get().unwrap();
        let new_exercise = new_exercise.to_new_exercise_model();
        let exercise = conn.create(&new_exercise)?;
        Ok(exercise)
    }

    fn updateExercise(
        context: &Context,
        updated_exercise: UpdatedExercise,
    ) -> Result<Exercise, database::Error> {
        let conn: &dyn ExerciseDao = &executor.context().pool.get().unwrap();
        let updated_exercise = updated_exercise.to_updated_exercise_model();
        let exercise = conn.update(&updated_exercise)?;
        Ok(exercise)
    }

    fn deleteExerciseById(context: &Context, id: String) -> Result<Exercise, database::Error> {
        let conn: &dyn ExerciseDao = &executor.context().pool.get().unwrap();
        let exercise = conn.delete_by_id(&id)?;
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
    fn graphql_crud_integration() {
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
        // Query for the exercise.
        ///////////////////////////////////////////////////////////////////////////////
        let find_by_id_request = create_graphql_request!(
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

        let response = make_test_graphql_request(&find_by_id_request).reply(&graphql_filter);
        let found_exercise: serde_json::Value = serde_json::from_slice(&response.body()).unwrap();
        let found_exercise: Exercise = found_exercise
            .as_object()
            .and_then(|map| map.get("data"))
            .and_then(|map| map.get("exercise"))
            .map(|map| map.to_string())
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .unwrap()
            .unwrap();

        assert_eq!(found_exercise, new_exercise);

        ///////////////////////////////////////////////////////////////////////////////
        // Update the exercise.
        ///////////////////////////////////////////////////////////////////////////////
        let request = create_graphql_request!(
            "mutation UpdateExercise($updatedExercise: UpdatedExercise!){
                updateExercise(updatedExercise: $updatedExercise) {
                    id,
                    title,
                    body,
                    topic,
                    createdOn,
                    modifiedOn
                }
            }"
            .replace("\n", " "),
            format!(
                "{{
                  \"updatedExercise\": {{
                    \"id\": \"{}\",
                    \"title\": \"The Amazing Albatross\"
                  }}
                }}",
                found_exercise.id.clone().unwrap()
            )
            .replace("\n", " ")
        );

        let response = make_test_graphql_request(&request).reply(&graphql_filter);
        let updated_exercise: serde_json::Value = serde_json::from_slice(&response.body()).unwrap();
        let updated_exercise: Exercise = updated_exercise
            .as_object()
            .and_then(|map| map.get("data"))
            .and_then(|map| map.get("updateExercise"))
            .map(|map| map.to_string())
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .unwrap()
            .unwrap();

        let expected_exercise = found_exercise;

        assert_eq!(expected_exercise.id, updated_exercise.id);
        assert_eq!(
            Some(String::from("The Amazing Albatross")),
            updated_exercise.title
        );
        assert_eq!(expected_exercise.body, updated_exercise.body);
        assert_eq!(expected_exercise.topic, updated_exercise.topic);
        assert_eq!(expected_exercise.created_on, updated_exercise.created_on);
        assert!(expected_exercise.modified_on <= updated_exercise.modified_on);

        ///////////////////////////////////////////////////////////////////////////////
        // Delete the exercise.
        ///////////////////////////////////////////////////////////////////////////////
        let request = create_graphql_request!(
            "mutation DeleteExerciseById($id: String!){
                deleteExerciseById(id: $id) {
                    id,
                    title,
                    body,
                    topic,
                    createdOn,
                    modifiedOn
                }
            }"
            .replace("\n", " "),
            format!("{{ \"id\": \"{}\" }}", updated_exercise.id.clone().unwrap())
        );

        let response = make_test_graphql_request(&request).reply(&graphql_filter);
        let deleted_exercise: serde_json::Value = serde_json::from_slice(&response.body()).unwrap();
        let deleted_exercise: Exercise = deleted_exercise
            .as_object()
            .and_then(|map| map.get("data"))
            .and_then(|map| map.get("deleteExerciseById"))
            .map(|map| map.to_string())
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .unwrap()
            .unwrap();

        assert_eq!(updated_exercise, deleted_exercise);

        // Verify the exercise was deleted.
        let response = make_test_graphql_request(&find_by_id_request).reply(&graphql_filter);
        let error: serde_json::Value = serde_json::from_slice(&response.body()).unwrap();
        let error = error
            .get("errors")
            .and_then(|map| map.get(0))
            .and_then(|map| map.get("extensions"))
            .unwrap()
            .as_object()
            .unwrap();

        assert!(error.contains_key("client_error"));
        assert_eq!(
            error
                .get("client_error")
                .and_then(serde_json::Value::as_str),
            Some("not_found")
        );
    }
}
