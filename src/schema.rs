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
