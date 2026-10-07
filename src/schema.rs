// @generated automatically by Diesel CLI.

diesel::table! {
    oauth_connections (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        provider -> Varchar,
        provider_user_id -> Text,
        access_token -> Text,
        refresh_token -> Nullable<Text>,
        expires_at -> Timestamptz,
        metadata -> Jsonb,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        token_hash -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        mail -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        password_hash -> Text,
        dashboard_map -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(oauth_connections -> users (user_id));
diesel::joinable!(refresh_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(oauth_connections, refresh_tokens, users,);
