const ALBATROSS_BODY: &'static str = "Albatrosses, of the biological family Diomedeidae, are
        large seabirds related to the procellariids, storm petrels, and diving petrels in the order
        Procellariiformes (the tubenoses).";

fn main() {
    use wikitype_api_graphql::database::models::Uuid;
    use wikitype_api_graphql::database::sql::sqlite;

    let conn = sqlite::new_in_memory();
    let exercise = sqlite::create_exercise(&conn, &Uuid::new(), "Albatross", ALBATROSS_BODY, None);

    let e = sqlite::get_exercise_by_title(&conn, "Albatross").expect("Test exercise not found.");
    assert_eq!(e, exercise);

    println!("{:#?}", exercise);
}
