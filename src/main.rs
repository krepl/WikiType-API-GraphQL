fn main() {
    use diesel::prelude::*;
    use dotenv::dotenv;
    use std::env;
    use wikitype_api_graphql::database::models::{ExerciseDao, NewExercise, UpdatedExercise, Uuid};

    const ALBATROSS_BODY: &'static str =
        "Albatrosses, of the biological family Diomedeidae, are large seabirds related to the \
         procellariids, storm petrels, and diving petrels in the order Procellariiformes (the \
         tubenoses).";

    // Connect to a Postgres database.
    //
    // NOTE: This database should already contain the `exercises` table. Otherwise, run the
    // migrations against the database.
    //
    // i.e.
    //     $ diesel migration run
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let dao: &dyn ExerciseDao = &PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to database {}.", database_url));

    // Create a new exercise.
    let id = Uuid::new();
    let new_exercise = NewExercise::new(&id, "Albatross", ALBATROSS_BODY, None);

    // Insert the new exercise into the database.
    let exercise = dao
        .create(&new_exercise)
        .expect("Failed to create Albatross exercise.");
    println!("{:#?}", exercise);

    // Create an updated exercise.
    let updated_exercise = UpdatedExercise {
        id: &exercise.id,
        title: Some("Alabatross new"),
        body: None,
        topic: Some("It's a topic!"),
    };

    // Update the exercise.
    let exercise = dao
        .update(&updated_exercise)
        .expect("Failed to create Albatross exercise.");
    println!("{:#?}", exercise);

    // Delete the exercise.
    let deleted_exercise = dao
        .delete_by_id(&exercise.id)
        .expect("Failed to create Albatross exercise.");
    assert_eq!(exercise, deleted_exercise);

    let exercise = dao.find_by_id(&exercise.id);
    assert_eq!(
        exercise,
        Err(wikitype_api_graphql::database::Error::SqlError(
            diesel::result::Error::NotFound
        ))
    );
}
