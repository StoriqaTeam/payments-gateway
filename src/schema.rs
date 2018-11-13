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
        callback_url -> Nullable<Varchar>,
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
        device_id -> Nullable<Varchar>,
        device_os -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts,
    users,
);
