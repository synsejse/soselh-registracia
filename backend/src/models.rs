use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;

use crate::schema::{admin_sessions, registrations, sessions, settings};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: i32,
    pub field_code: String,
    pub field_name: String,
    pub session_date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub max_capacity: i32,
    pub turnus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = registrations)]
pub struct Registration {
    pub id: i32,
    pub session_id: i32,
    pub student_first_name: String,
    pub student_last_name: String,
    pub guardian_first_name: String,
    pub guardian_last_name: String,
    pub guardian_phone: String,
    pub guardian_email: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = registrations)]
pub struct NewRegistration {
    pub session_id: i32,
    pub student_first_name: String,
    pub student_last_name: String,
    pub guardian_first_name: String,
    pub guardian_last_name: String,
    pub guardian_phone: String,
    pub guardian_email: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub key_name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateRegistrationRequest {
    pub session_id: i32,
    pub student_first_name: String,
    pub student_last_name: String,
    pub guardian_first_name: String,
    pub guardian_last_name: String,
    pub guardian_phone: String,
    pub guardian_email: String,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SessionWithAvailability {
    pub id: i32,
    pub field_code: String,
    pub field_name: String,
    pub session_date: String,
    pub start_time: String,
    pub end_time: String,
    pub max_capacity: i32,
    pub turnus: i32,
    pub available_spots: i32,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegistrationResponse {
    pub id: i32,
    pub session: Session,
    pub student_first_name: String,
    pub student_last_name: String,
    pub guardian_first_name: String,
    pub guardian_last_name: String,
    pub guardian_phone: String,
    pub guardian_email: String,
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = admin_sessions)]
pub struct AdminSession {
    pub session_token: String,
    pub created_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = admin_sessions)]
pub struct NewAdminSession {
    pub session_token: String,
    pub expires_at: Option<NaiveDateTime>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AdminLoginRequest {
    pub password: String,
}
