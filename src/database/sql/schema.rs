table! {
    exercises (id) {
        id -> Varchar,
        title -> Varchar,
        body -> Text,
        topic -> Nullable<Varchar>,
        created_on -> Timestamp,
        modified_on -> Timestamp,
    }
}
