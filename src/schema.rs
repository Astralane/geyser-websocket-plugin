// @generated automatically by Diesel CLI.

diesel::table! {
    chain_transactions (id) {
        id -> Int4,
        signature -> Varchar,
        fee -> Int8,
        slot -> Int4,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        from_user -> Int4,
        to_user -> Int4,
        amount -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        balance -> Int4,
        removed -> Bool,
        password -> Varchar,
        session_token -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(chain_transactions, transactions, users,);
