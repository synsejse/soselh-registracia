// @generated automatically by Diesel CLI.

diesel::table! {
    admin_sessions (session_token) {
        #[max_length = 36]
        session_token -> Varchar,
        created_at -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
    }
}

diesel::table! {
    registrations (id) {
        id -> Integer,
        session_id -> Integer,
        #[max_length = 100]
        student_first_name -> Varchar,
        #[max_length = 100]
        student_last_name -> Varchar,
        #[max_length = 100]
        guardian_first_name -> Varchar,
        #[max_length = 100]
        guardian_last_name -> Varchar,
        #[max_length = 20]
        guardian_phone -> Varchar,
        #[max_length = 255]
        guardian_email -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Integer,
        #[max_length = 3]
        field_code -> Varchar,
        #[max_length = 100]
        field_name -> Varchar,
        session_date -> Date,
        start_time -> Time,
        end_time -> Time,
        max_capacity -> Integer,
        turnus -> Integer,
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

diesel::joinable!(registrations -> sessions (session_id));

diesel::allow_tables_to_appear_in_same_query!(admin_sessions, registrations, sessions, settings,);
