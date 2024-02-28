// @generated automatically by Diesel CLI.

diesel::table! {
    employees (id) {
        id -> Int4,
        name -> Varchar,
        created_at -> Timestamp,
    }
}
