use super::sql::schema::exercises;

use crate::database;

/// A [data access object] for exercises.
///
/// [data access object]: https://en.wikipedia.org/wiki/Data_access_object
pub trait ExerciseDao {
    fn create_exercise<'a>(
        &self,
        title: &'a str,
        body: &'a str,
        topic: Option<&'a str>,
    ) -> database::Result<Exercise>;

    fn get_exercise_by_id<'a>(&self, id: &'a str) -> database::Result<Exercise>;

    fn get_exercise_by_title<'a>(&self, title: &'a str) -> database::Result<Exercise>;

    // TODO: finish CRUD operations
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

/// Representation for exercises.
#[derive(Queryable, Debug, Eq, PartialEq)]
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

/// TODO
#[derive(Insertable)]
#[table_name = "exercises"]
pub struct NewExercise<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub topic: Option<&'a str>,
    // TODO
    //created_on: ...,
    //modified_on: ...,
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
