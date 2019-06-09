use database::models::{Exercise, ExerciseDao, NewExerciseBuilder, UpdatedExerciseBuilder};
use diesel::prelude::*;
use diesel::r2d2;
use dotenv::dotenv;
use std::env;
use std::thread;
use wikitype_api_graphql::database;

const ALBATROSS_BODY: &'static str =
    "Albatrosses, of the biological family Diomedeidae, are large seabirds related to the \
     procellariids, storm petrels, and diving petrels in the order Procellariiformes (the \
     tubenoses).";

fn main() {
    // Connect to a Postgres database.
    //
    // NOTE: This database should already contain the `exercises` table. Otherwise, run the
    // migrations against the database.
    //
    // i.e.
    //     $ diesel migration run
    dotenv().ok();

    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let dao: &dyn ExerciseDao = &PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to database {}.", database_url));

    // Create a new exercise.
    let new_exercise = NewExerciseBuilder::new()
        .title("Albatross")
        .body(ALBATROSS_BODY)
        .build();

    // Insert the new exercise into the database.
    let exercise = dao
        .create(&new_exercise)
        .expect("Failed to create Albatross exercise.");
    println!("{:#?}", exercise);

    // Create an updated exercise.
    let updated_exercise = UpdatedExerciseBuilder::new(&exercise)
        .title("Alabatross new")
        .topic(Some("It's a topic!"))
        .build();

    // Update the exercise.
    let exercise = dao
        .update(&updated_exercise)
        .expect("Failed to create Albatross exercise.");
    println!("{:#?}", exercise);

    // Demonstrate use of r2d2::PooledConnection<M> as an ExerciseDao.
    let manager: r2d2::ConnectionManager<PgConnection> = r2d2::ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder().max_size(10).build(manager).unwrap();

    let join_handles: Vec<thread::JoinHandle<database::Result<Exercise>>> = (0..20)
        .map(|_| {
            let pool = pool.clone();
            let exercise = exercise.clone();
            thread::spawn(move || {
                let conn: &dyn ExerciseDao = &pool.get().unwrap();
                conn.find_by_id(&exercise.id)
            })
        })
        .collect();

    for jh in join_handles {
        assert_eq!(jh.join().unwrap(), Ok(exercise.clone()));
    }

    // Delete the exercise.
    let deleted_exercise = dao
        .delete_by_id(&exercise.id)
        .expect("Failed to create Albatross exercise.");
    assert_eq!(exercise, deleted_exercise);

    let exercise = dao.find_by_id(&exercise.id);
    assert_eq!(
        exercise,
        Err(database::Error::SqlError(diesel::result::Error::NotFound))
    );
}
