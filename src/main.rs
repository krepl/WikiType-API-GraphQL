const ALBATROSS_BODY: &'static str =
    "Albatrosses, of the biological family Diomedeidae, are large seabirds related to the \
     procellariids, storm petrels, and diving petrels in the order Procellariiformes (the \
     tubenoses).";

fn main() {
    use wikitype_api_graphql::database::models::ExerciseDao;
    use wikitype_api_graphql::database::sql::postgres::PostgresExerciseDao;

    let dao = PostgresExerciseDao::new();

    let exercise = dao
        .create_exercise("Albatross", ALBATROSS_BODY, None)
        .expect("Failed to create Albatross exercise.");

    println!("{:#?}", exercise);
}
