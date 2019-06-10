use crate::database::sql::schema::exercises;

use std::fmt;

/// A WikiType typing exercise.
#[derive(juniper::GraphQLObject, Queryable, Debug, Eq, PartialEq, Clone)]
pub struct Exercise {
    /// UUID string.
    pub id: String,

    /// Title of the exercise.
    pub title: String,

    /// Content of the exercise.
    pub body: String,

    /// Optional topic describing the general exercise category.
    ///
    /// See <https://en.wikipedia.org/wiki/Portal:Contents/Portals> for an idea.
    pub topic: Option<String>,

    /// Date and time of creation.
    pub created_on: chrono::NaiveDateTime,

    /// Date and time of the last modification.
    pub modified_on: chrono::NaiveDateTime,
}

impl fmt::Display for Exercise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Type for creating a new `Exercise`.
#[derive(Insertable)]
#[table_name = "exercises"]
pub struct NewExercise {
    id: String,
    pub title: String,
    pub body: String,
    pub topic: Option<String>,
    created_on: chrono::NaiveDateTime,
    modified_on: chrono::NaiveDateTime,
}

impl NewExercise {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

/// Type for creating a `NewExercise`.
///
/// # Examples
///
/// ```
/// use wikitype_api::models::NewExerciseBuilder;
///
/// // Create an updated exercise.
/// let new_exercise = NewExerciseBuilder::new()
///     .title("Albatross")
///     .body("Albatross body")
///     .topic("It's a topic!")
///     .build();
///
/// assert_eq!(new_exercise.title, "Albatross");
/// assert_eq!(new_exercise.body, "Albatross body");
/// assert_eq!(new_exercise.topic, Some(String::from("It's a topic!")));
/// ```
pub struct NewExerciseBuilder<'a> {
    id: String,
    title: Option<&'a str>,
    body: Option<&'a str>,
    topic: Option<String>,
}

impl<'a> NewExerciseBuilder<'a> {
    pub fn new() -> NewExerciseBuilder<'a> {
        NewExerciseBuilder {
            id: Uuid::new().to_string(),
            title: None,
            body: None,
            topic: None,
        }
    }

    pub fn title(&mut self, title: &'a str) -> &mut NewExerciseBuilder<'a> {
        self.title = Some(title);
        self
    }

    pub fn body(&mut self, body: &'a str) -> &mut NewExerciseBuilder<'a> {
        self.body = Some(body);
        self
    }

    pub fn topic(&mut self, topic: &Option<String>) -> &mut NewExerciseBuilder<'a> {
        self.topic = topic.clone();
        self
    }

    pub fn build(&mut self) -> NewExercise {
        let title = self.title.expect("Missing exercise title.").to_string();
        let body = self.body.expect("Missing exercise body.").to_string();
        let created_on = chrono::Utc::now().naive_utc();
        let modified_on = created_on;
        NewExercise {
            id: self.id.clone(),
            title,
            body,
            topic: self.topic.clone(),
            created_on,
            modified_on,
        }
    }
}

/// Type for updating an `Exercise`.
#[derive(AsChangeset, Identifiable, Clone)]
#[table_name = "exercises"]
pub struct UpdatedExercise<'a> {
    id: String,
    pub title: Option<&'a str>,
    pub body: Option<&'a str>,
    pub topic: Option<Option<&'a str>>,
    modified_on: chrono::NaiveDateTime,
}

impl<'a> UpdatedExercise<'a> {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

/// Type for creating an `UpdatedExercise`.
///
/// # Examples
///
/// ```
/// use wikitype_api::models::{Exercise, Uuid, UpdatedExerciseBuilder};
/// use chrono::NaiveDateTime;
///
/// // Create an initial exercise.
/// let exercise = Exercise {
///     id: Uuid::new().to_string(),
///     title: String::from(""),
///     body: String::from(""),
///     topic: None,
///     created_on: NaiveDateTime::from_timestamp(0, 0),
///     modified_on: NaiveDateTime::from_timestamp(0, 0),
/// };
///
/// // Create an updated exercise.
/// let updated_exercise = UpdatedExerciseBuilder::new(&exercise)
///     .title("Alabatross new")
///     .topic(Some("It's a topic!"))
///     .build();
///
/// assert_eq!(exercise.id, updated_exercise.get_id());
/// assert_eq!(None, updated_exercise.body);
/// ```
pub struct UpdatedExerciseBuilder<'a> {
    exercise: UpdatedExercise<'a>,
}

impl<'a> UpdatedExerciseBuilder<'a> {
    pub fn new(exercise: &Exercise) -> UpdatedExerciseBuilder<'a> {
        UpdatedExerciseBuilder {
            exercise: UpdatedExercise {
                id: exercise.id.clone(),
                title: None,
                body: None,
                topic: None,
                modified_on: exercise.modified_on,
            },
        }
    }

    pub fn title(&mut self, title: &'a str) -> &mut UpdatedExerciseBuilder<'a> {
        self.exercise.title = Some(title);
        self
    }

    pub fn body(&mut self, body: &'a str) -> &mut UpdatedExerciseBuilder<'a> {
        self.exercise.body = Some(body);
        self
    }

    pub fn topic(&mut self, topic: Option<&'a str>) -> &mut UpdatedExerciseBuilder<'a> {
        self.exercise.topic = Some(topic);
        self
    }

    pub fn build(&mut self) -> UpdatedExercise<'a> {
        self.exercise.modified_on = chrono::Utc::now().naive_utc();
        self.exercise.clone()
    }
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
