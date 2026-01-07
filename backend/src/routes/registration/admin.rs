use bcrypt::verify;
use rocket::State;
use rocket::http::{ContentType, Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::prelude::*;
use rust_xlsxwriter::Workbook;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use crate::AppState;
use crate::db::RegistrationDB;
use crate::models::{
    AdminLoginRequest, NewAdminSession, Registration, RegistrationResponse, Session,
};
use crate::schema::{admin_sessions, registrations, sessions, settings};

// Helper function to check if admin is authenticated
async fn is_admin_authenticated(
    cookies: &CookieJar<'_>,
    db: &mut Connection<RegistrationDB>,
) -> bool {
    if let Some(cookie) = cookies.get("admin_auth") {
        let token = cookie.value();
        admin_sessions::table
            .find(token)
            .count()
            .get_result::<i64>(db)
            .await
            .unwrap_or(0)
            > 0
    } else {
        false
    }
}

// Admin login endpoint
#[post("/admin/login", format = "json", data = "<login>")]
pub async fn admin_login(
    mut db: Connection<RegistrationDB>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    login: Json<AdminLoginRequest>,
) -> Result<Status, Status> {
    if verify(&login.password, &state.presenter_password_hash).unwrap_or(false) {
        let token = Uuid::new_v4().to_string();
        let new_session = NewAdminSession {
            session_token: token.clone(),
            expires_at: None,
            ip_address: None,
        };

        diesel::insert_into(admin_sessions::table)
            .values(&new_session)
            .execute(&mut db)
            .await
            .map_err(|e| {
                eprintln!("Error creating admin session: {}", e);
                Status::InternalServerError
            })?;

        let mut cookie = Cookie::new("admin_auth", token);
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_path("/");
        cookies.add(cookie);
        Ok(Status::Ok)
    } else {
        // Clear any existing invalid cookie
        cookies.remove(Cookie::from("admin_auth"));
        Err(Status::Unauthorized)
    }
}

// Admin logout endpoint
#[post("/admin/logout")]
pub async fn admin_logout(
    mut db: Connection<RegistrationDB>,
    cookies: &CookieJar<'_>,
) -> Result<Status, Status> {
    if let Some(cookie) = cookies.get("admin_auth") {
        let token = cookie.value();
        diesel::delete(admin_sessions::table.find(token))
            .execute(&mut db)
            .await
            .ok();
        cookies.remove(Cookie::from("admin_auth"));
    }
    Ok(Status::Ok)
}

// Check if admin is authenticated
#[get("/admin/check")]
pub async fn admin_check(
    mut db: Connection<RegistrationDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<bool>, Status> {
    let authenticated = is_admin_authenticated(cookies, &mut db).await;
    Ok(Json(authenticated))
}

// Route to get all registrations (admin view) - requires authentication
#[get("/admin/registrations")]
pub async fn get_all_registrations(
    mut db: Connection<RegistrationDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<RegistrationResponse>>, Status> {
    // Check authentication
    if !is_admin_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }
    let all_registrations = registrations::table
        .inner_join(sessions::table.on(registrations::session_id.eq(sessions::id)))
        .select((Registration::as_select(), Session::as_select()))
        .load::<(Registration, Session)>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading registrations: {}", e);
            Status::InternalServerError
        })?;

    let response: Vec<RegistrationResponse> = all_registrations
        .into_iter()
        .map(|(reg, session)| RegistrationResponse {
            id: reg.id,
            session,
            student_first_name: reg.student_first_name,
            student_last_name: reg.student_last_name,
            guardian_first_name: reg.guardian_first_name,
            guardian_last_name: reg.guardian_last_name,
            guardian_phone: reg.guardian_phone,
            guardian_email: reg.guardian_email,
            created_at: reg
                .created_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        })
        .collect();

    Ok(Json(response))
}

// Route to export registrations to Excel - requires authentication
#[get("/admin/registrations/export")]
pub async fn export_registrations_excel(
    mut db: Connection<RegistrationDB>,
    cookies: &CookieJar<'_>,
) -> Result<(ContentType, Vec<u8>), Status> {
    // Check authentication
    if !is_admin_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }

    let all_registrations = registrations::table
        .inner_join(sessions::table.on(registrations::session_id.eq(sessions::id)))
        .select((Registration::as_select(), Session::as_select()))
        .load::<(Registration, Session)>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading registrations: {}", e);
            Status::InternalServerError
        })?;

    // Create Excel
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Headers
    let headers = [
        "ID",
        "Meno študenta",
        "Priezvisko študenta",
        "Meno zákonného zástupcu",
        "Priezvisko zákonného zástupcu",
        "Email",
        "Telefón",
        "Dátum a čas",
        "Odbor",
        "Vytvorené",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet
            .write_string(0, col as u16, *header)
            .map_err(|_| Status::InternalServerError)?;
    }

    for (i, (reg, session)) in all_registrations.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet
            .write_number(row, 0, reg.id as f64)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 1, &reg.student_first_name)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 2, &reg.student_last_name)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 3, &reg.guardian_first_name)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 4, &reg.guardian_last_name)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 5, &reg.guardian_email)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 6, &reg.guardian_phone)
            .map_err(|_| Status::InternalServerError)?;

        let session_time = format!(
            "{} {} - {}",
            session.session_date, session.start_time, session.end_time
        );
        worksheet
            .write_string(row, 7, &session_time)
            .map_err(|_| Status::InternalServerError)?;
        worksheet
            .write_string(row, 8, &session.field_code)
            .map_err(|_| Status::InternalServerError)?;

        let created_at = reg
            .created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();
        worksheet
            .write_string(row, 9, &created_at)
            .map_err(|_| Status::InternalServerError)?;
    }

    worksheet.autofit();

    let buf = workbook.save_to_buffer().map_err(|e| {
        eprintln!("Error saving excel buffer: {}", e);
        Status::InternalServerError
    })?;

    Ok((
        ContentType::new(
            "application",
            "vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        buf,
    ))
}

// Route to toggle registration status - requires authentication
#[post("/admin/toggle")]
pub async fn toggle_registration(
    mut db: Connection<RegistrationDB>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<bool>, Status> {
    // Check authentication
    if !is_admin_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }
    let current =
        std::sync::atomic::AtomicBool::load(&state.registration_enabled, Ordering::Relaxed);
    let new_value = !current;

    diesel::update(settings::table.find("registration_enabled"))
        .set(settings::value.eq(if new_value { "true" } else { "false" }))
        .execute(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error updating registration status: {}", e);
            Status::InternalServerError
        })?;

    state
        .registration_enabled
        .store(new_value, Ordering::Relaxed);

    Ok(Json(new_value))
}
