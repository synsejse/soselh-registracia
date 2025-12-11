use bcrypt::verify;
use chrono::{Duration, Utc};
use rand::{Rng, thread_rng};
use rocket::State;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::prelude::*;
use uuid::Uuid;

use crate::AppState;
use crate::db::VotingDB;
use crate::models::{
    AdminStats, CandidateResult, LotteryWinner, NewPresenterSession, UpdateVotingStatusRequest,
    VotingSession,
};
use crate::schema::{presenter_sessions, settings, voting_sessions};

#[derive(Debug, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PresenterLoginRequest {
    pub password: String,
}

async fn is_presenter_authenticated(
    cookies: &CookieJar<'_>,
    db: &mut Connection<VotingDB>,
) -> bool {
    if let Some(cookie) = cookies.get("presenter_auth") {
        let token = cookie.value();
        let session = presenter_sessions::table
            .find(token)
            .first::<crate::models::PresenterSession>(db)
            .await;

        match session {
            Ok(s) => {
                if s.expires_at
                    .is_some_and(|expires| expires < Utc::now().naive_utc())
                {
                    // Session expired, clean it up
                    let _ = diesel::delete(presenter_sessions::table.find(token))
                        .execute(db)
                        .await;
                    return false;
                }
                true
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

#[post("/presenter/login", format = "json", data = "<login>")]
pub async fn presenter_login(
    mut db: Connection<VotingDB>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    login: Json<PresenterLoginRequest>,
) -> Result<Status, Status> {
    if verify(&login.password, &state.presenter_password_hash).unwrap_or(false) {
        let token = Uuid::new_v4().to_string();
        // Set session expiry to 24 hours
        let expires = Utc::now().naive_utc() + Duration::hours(24);

        let new_session = NewPresenterSession {
            session_token: token.clone(),
            expires_at: Some(expires),
        };

        diesel::insert_into(presenter_sessions::table)
            .values(&new_session)
            .execute(&mut db)
            .await
            .map_err(|e| {
                eprintln!("Error creating session: {}", e);
                Status::InternalServerError
            })?;

        let mut cookie = Cookie::new("presenter_auth", token);
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_path("/");
        cookies.add(cookie);
        Ok(Status::Ok)
    } else {
        Err(Status::Unauthorized)
    }
}

#[post("/presenter/logout")]
pub async fn presenter_logout(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
) -> Result<Status, Status> {
    if let Some(cookie) = cookies.get("presenter_auth") {
        let token = cookie.value();
        diesel::delete(presenter_sessions::table.find(token))
            .execute(&mut db)
            .await
            .ok();
        cookies.remove(Cookie::from("presenter_auth"));
    }
    Ok(Status::Ok)
}

#[get("/presenter/check")]
pub async fn presenter_check(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<bool>, Status> {
    let ok = is_presenter_authenticated(cookies, &mut db).await;
    Ok(Json(ok))
}

#[get("/admin/status")]
pub async fn get_status(
    mut db: Connection<VotingDB>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<bool>, Status> {
    if !is_presenter_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }

    let enabled = std::sync::atomic::AtomicBool::load(
        &state.voting_enabled,
        std::sync::atomic::Ordering::Relaxed,
    );
    Ok(Json(enabled))
}

#[post("/admin/status", format = "json", data = "<status_request>")]
pub async fn set_voting_status(
    mut db: Connection<VotingDB>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    status_request: Json<UpdateVotingStatusRequest>,
) -> Result<Status, Status> {
    if !is_presenter_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }

    let new_value = if status_request.action == "start" {
        "true"
    } else {
        "false"
    };

    diesel::update(settings::table.find("voting_enabled"))
        .set(settings::value.eq(new_value))
        .execute(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error updating voting status: {}", e);
            Status::InternalServerError
        })?;

    state.voting_enabled.store(
        status_request.action == "start",
        std::sync::atomic::Ordering::Relaxed,
    );

    let _ = state.tx.send(status_request.action == "start");

    Ok(Status::Ok)
}

#[get("/admin/stats")]
pub async fn get_stats(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<AdminStats>, Status> {
    if !is_presenter_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }

    use crate::schema::votes;

    let voted_count: i64 = votes::table
        .count()
        .get_result(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error getting vote stats: {}", e);
            Status::InternalServerError
        })?;

    let total_sessions: i64 = voting_sessions::table
        .count()
        .get_result(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error getting session stats: {}", e);
            Status::InternalServerError
        })?;

    let unvoted_count = total_sessions - voted_count;

    Ok(Json(AdminStats {
        voted: voted_count,
        unvoted: unvoted_count,
    }))
}

#[get("/admin/results")]
pub async fn get_results(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<CandidateResult>>, Status> {
    if !is_presenter_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }

    use crate::schema::{candidates, votes};
    use diesel::dsl::count;

    let results = candidates::table
        .left_join(votes::table)
        .group_by(candidates::id)
        .select((candidates::name, count(votes::id.nullable())))
        .load::<(String, i64)>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading results: {}", e);
            Status::InternalServerError
        })?
        .into_iter()
        .map(|(name, votes)| CandidateResult { name, votes })
        .collect();

    Ok(Json(results))
}

#[get("/admin/lottery")]
pub async fn pick_winner(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<LotteryWinner>, Status> {
    if !is_presenter_authenticated(cookies, &mut db).await {
        return Err(Status::Unauthorized);
    }

    let sessions = voting_sessions::table
        .load::<VotingSession>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading sessions: {}", e);
            Status::InternalServerError
        })?;

    if sessions.is_empty() {
        return Err(Status::NotFound);
    }

    let mut rng = thread_rng();
    let winner_idx = rng.gen_range(0..sessions.len());
    let winner = &sessions[winner_idx];

    Ok(Json(LotteryWinner {
        name: winner.display_name.clone(),
        voter_id: winner.voter_id.clone(),
    }))
}
