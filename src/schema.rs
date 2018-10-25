table! {
    accounts (id) {
        id -> Uuid,
        user_id -> Int4,
        currency -> Varchar,
        account_address -> Varchar,
        name -> Varchar,
        balance -> Numeric,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
