// @generated automatically by Diesel CLI.

diesel::table! {
    division_updates (id) {
        id -> Int4,
        division_id -> Int4,
        publication_updated -> Text,
    }
}

diesel::table! {
    divisions (id) {
        id -> Int4,
        division_id -> Int4,
        discord_thread_id -> Int8,
    }
}

diesel::joinable!(division_updates -> divisions (division_id));

diesel::allow_tables_to_appear_in_same_query!(
    division_updates,
    divisions,
);
