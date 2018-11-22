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
    devices (device_id, user_id) {
        device_id -> Varchar,
        device_os -> Varchar,
        user_id -> Int4,
        public_key -> Varchar,
        last_timestamp -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    devices_tokens (id) {
        id -> Uuid,
        device_id -> Varchar,
        device_os -> Varchar,
        user_id -> Int4,
        public_key -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    templates (id) {
        id -> Int4,
        name -> Varchar,
        data -> Varchar,
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
        device_id -> Nullable<Varchar>,
        device_os -> Nullable<Varchar>,
    }
}

joinable!(devices -> users (user_id));
joinable!(devices_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(accounts, devices, devices_tokens, templates, users,);
