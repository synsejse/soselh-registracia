use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::AppState;
use crate::db::RegistrationDB;
use crate::models::{CreateRegistrationRequest, NewRegistration, Session, SessionWithAvailability};
use crate::schema::{registrations, sessions};

// Route to get all available sessions with availability info
#[get("/sessions")]
pub async fn get_sessions(
    mut db: Connection<RegistrationDB>,
) -> Result<Json<Vec<SessionWithAvailability>>, Status> {
    let all_sessions = sessions::table
        .load::<Session>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading sessions: {}", e);
            Status::InternalServerError
        })?;

    let mut sessions_with_availability = Vec::new();

    for session in all_sessions {
        // Count registrations for this session
        let registration_count: i64 = registrations::table
            .filter(registrations::session_id.eq(session.id))
            .filter(registrations::confirmed.eq(true))
            .count()
            .get_result(&mut db)
            .await
            .unwrap_or(0);

        let available_spots = session.max_capacity - registration_count as i32;

        sessions_with_availability.push(SessionWithAvailability {
            id: session.id,
            field_code: session.field_code.clone(),
            field_name: session.field_name.clone(),
            session_date: session.session_date.format("%Y-%m-%d").to_string(),
            start_time: session.start_time.format("%H:%M").to_string(),
            end_time: session.end_time.format("%H:%M").to_string(),
            max_capacity: session.max_capacity,
            turnus: session.turnus,
            available_spots,
        });
    }

    Ok(Json(sessions_with_availability))
}

// Route to create a new registration
#[post("/register", format = "json", data = "<registration_request>")]
pub async fn create_registration(
    mut db: Connection<RegistrationDB>,
    state: &State<AppState>,
    registration_request: Json<CreateRegistrationRequest>,
) -> Result<Json<i32>, Status> {
    // Check if registration is enabled
    if !AtomicBool::load(&state.registration_enabled, Ordering::Relaxed) {
        return Err(Status::PreconditionFailed); // Registration not enabled
    }

    // Validate session exists and has capacity
    let session = sessions::table
        .find(registration_request.session_id)
        .first::<Session>(&mut db)
        .await
        .map_err(|_| Status::NotFound)?;

    // Count current registrations
    let registration_count: i64 = registrations::table
        .filter(registrations::session_id.eq(session.id))
        .filter(registrations::confirmed.eq(true))
        .count()
        .get_result(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error counting registrations: {}", e);
            Status::InternalServerError
        })?;

    if registration_count >= session.max_capacity as i64 {
        return Err(Status::Conflict); // Session is full
    }

    // Create new registration
    let new_registration = NewRegistration {
        session_id: registration_request.session_id,
        student_first_name: registration_request.student_first_name.clone(),
        student_last_name: registration_request.student_last_name.clone(),
        guardian_first_name: registration_request.guardian_first_name.clone(),
        guardian_last_name: registration_request.guardian_last_name.clone(),
        guardian_phone: registration_request.guardian_phone.clone(),
        guardian_email: registration_request.guardian_email.clone(),
    };

    let result = diesel::insert_into(registrations::table)
        .values(&new_registration)
        .execute(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error creating registration: {}", e);
            Status::InternalServerError
        })?;

    if result > 0 {
        // Get the ID of the inserted registration
        let registration_id = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
            "LAST_INSERT_ID()",
        ))
        .get_result::<i32>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error getting registration ID: {}", e);
            Status::InternalServerError
        })?;

        Ok(Json(registration_id))
    } else {
        Err(Status::InternalServerError)
    }
}

// Route to check registration status
#[get("/status")]
pub async fn get_registration_status(state: &State<AppState>) -> Json<bool> {
    Json(AtomicBool::load(
        &state.registration_enabled,
        Ordering::Relaxed,
    ))
}
