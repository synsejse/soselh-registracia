// @generated automatically by Diesel CLI.

diesel::table! {
    candidates (id) {
        id -> Integer,
        #[max_length = 150]
        name -> Varchar,
    }
}

diesel::table! {
    settings (key_name) {
        #[max_length = 50]
        key_name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    votes (id) {
        id -> Integer,
        #[max_length = 64]
        session_token -> Varchar,
        candidate_id -> Integer,
        voted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    voting_sessions (session_token) {
        #[max_length = 64]
        session_token -> Varchar,
        #[max_length = 100]
        display_name -> Varchar,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        #[max_length = 5]
        voter_id -> Varchar,
    }
}

diesel::joinable!(votes -> candidates (candidate_id));
diesel::joinable!(votes -> voting_sessions (session_token));

diesel::allow_tables_to_appear_in_same_query!(candidates, settings, votes, voting_sessions,);
