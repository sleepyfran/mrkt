// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Nullable<Integer>,
        name -> Text,
        owner_id -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    sessions (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        token -> Text,
        created_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Nullable<Integer>,
        owner_id -> Integer,
        account_id -> Integer,
        transaction_type -> Integer,
        ticker_symbol -> Text,
        transaction_date -> Text,
        quantity -> Float,
        price_per_share -> Float,
        fees -> Float,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        hashed_password -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(accounts -> users (owner_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(transactions -> accounts (account_id));
diesel::joinable!(transactions -> users (owner_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    sessions,
    transactions,
    users,
);
