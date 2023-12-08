// @generated automatically by Diesel CLI.

diesel::table! {
    tasks (id) {
        id -> Unsigned<Integer>,
        #[max_length = 100]
        task_id -> Varchar,
        params -> Text,
        result -> Text,
        starts_at -> Nullable<Timestamp>,
        ends_at -> Nullable<Timestamp>,
        #[max_length = 200]
        callback_url -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        generation_params -> Text,
    }
}
