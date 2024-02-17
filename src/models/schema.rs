// @generated automatically by Diesel CLI.

diesel::table! {
    clients (id) {
        id -> Int4,
        account_limit -> Int4,
        balance -> Int4,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        client_id -> Int4,
        amount -> Int4,
        #[max_length = 1]
        transaction_type -> Varchar,
        details -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    last_transactions (id) {
        id -> Int4,
        client_id -> Int4,
        amount -> Int4,
        #[max_length = 1]
        transaction_type -> Varchar,
        details -> Text,
        created_at -> Timestamp,
        balance -> Int4,
        account_limit -> Int4,
    }
}

diesel::joinable!(transactions -> clients (client_id));
diesel::joinable!(last_transactions -> clients (client_id));

diesel::allow_tables_to_appear_in_same_query!(clients, transactions,);

diesel::allow_tables_to_appear_in_same_query!(clients, last_transactions,);
