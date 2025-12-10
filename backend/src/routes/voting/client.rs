use diesel::result::Error;
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use rocket::State;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::prelude::*;
use std::sync::atomic::AtomicBool;
use uuid::Uuid;

use crate::AppState;
use crate::db::VotingDB;
use crate::models::{
    Candidate, CastVoteRequest, CreateSessionRequest, NewVote, NewVotingSession,
    SessionInfoResponse, VotingSession, VotingStatusResponse,
};
use crate::schema::{candidates, votes, voting_sessions};

fn generate_voter_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

// Route to create a new voting session
#[post("/session", format = "json", data = "<session_request>")]
pub async fn create_session(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
    session_request: Json<CreateSessionRequest>,
) -> Result<Json<SessionInfoResponse>, Status> {
    let token = Uuid::new_v4().to_string();
    let voter_id = generate_voter_id();

    let new_session = NewVotingSession {
        session_token: token.clone(),
        display_name: session_request.name.clone(),
        ip_address: None,
        voter_id: voter_id.clone(),
    };

    diesel::insert_into(voting_sessions::table)
        .values(&new_session)
        .execute(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error creating session: {}", e);
            Status::InternalServerError
        })?;

    let mut cookie = Cookie::new("session_token", token);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok(Json(SessionInfoResponse {
        voter_id,
        name: session_request.name.clone(),
    }))
}

// Route to get current session info
#[get("/session")]
pub async fn get_session_info(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
) -> Result<Json<SessionInfoResponse>, Status> {
    let token_val = cookies
        .get("session_token")
        .map(|c| c.value())
        .ok_or(Status::Unauthorized)?;

    let session = voting_sessions::table
        .find(token_val)
        .first::<VotingSession>(&mut db)
        .await
        .map_err(|_| Status::Unauthorized)?;

    Ok(Json(SessionInfoResponse {
        voter_id: session.voter_id,
        name: session.display_name,
    }))
}

// Route to get candidates
#[get("/candidates")]
pub async fn get_candidates(mut db: Connection<VotingDB>) -> Result<Json<Vec<Candidate>>, Status> {
    let results = candidates::table
        .load::<Candidate>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading candidates: {}", e);
            Status::InternalServerError
        })?;

    Ok(Json(results))
}

// Route to cast a vote
#[post("/vote", format = "json", data = "<vote_request>")]
pub async fn cast_vote(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    vote_request: Json<CastVoteRequest>,
) -> Result<Status, Status> {
    // Check if voting is enabled
    if !AtomicBool::load(&state.voting_enabled, std::sync::atomic::Ordering::Relaxed) {
        return Err(Status::PreconditionFailed); // Voting not started
    }

    let token_val = cookies
        .get("session_token")
        .map(|c| c.value())
        .ok_or(Status::Unauthorized)?;

    let new_vote = NewVote {
        session_token: token_val.to_string(),
        candidate_id: vote_request.candidate_id,
    };

    let result = diesel::insert_into(votes::table)
        .values(&new_vote)
        .execute(&mut db)
        .await;

    match result {
        Ok(_) => Ok(Status::Created),
        Err(Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
            Err(Status::Conflict) // Already voted
        }
        Err(e) => {
            eprintln!("Error casting vote: {}", e);
            Err(Status::InternalServerError)
        }
    }
}

// Route to check status (voting enabled + user voted)
#[get("/status")]
pub async fn get_vote_status(
    mut db: Connection<VotingDB>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Json<VotingStatusResponse>, Status> {
    let voting_enabled =
        AtomicBool::load(&state.voting_enabled, std::sync::atomic::Ordering::Relaxed);

    let mut has_voted = false;

    if let Some(token_cookie) = cookies.get("session_token") {
        use crate::schema::votes::dsl::{session_token, votes as votes_table};

        let count: i64 = votes_table
            .filter(session_token.eq(token_cookie.value()))
            .count()
            .get_result(&mut db)
            .await
            .unwrap_or(0);

        has_voted = count > 0;
    }

    Ok(Json(VotingStatusResponse {
        ready: voting_enabled,
        has_voted,
    }))
}
