// @generated automatically by Diesel CLI.

diesel::table! {
    users (user_id) {
        user_id -> Integer,
        username -> Text,
        email -> Text,
        password -> Text,
    }
}
