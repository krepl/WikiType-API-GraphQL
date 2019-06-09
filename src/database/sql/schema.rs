table! {
    exercises (id) {
        id -> Varchar,
        title -> Varchar,
        body -> Text,
        topic -> Nullable<Varchar>,
        created_on -> Int8,
        modified_on -> Int8,
    }
}
