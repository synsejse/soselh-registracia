use rand::{Rng, thread_rng};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use rocket_db_pools::diesel::prelude::*;

use crate::db::VotingDB;
use crate::models::{
    Candidate, CandidateResult, LotteryWinner, UpdateVotingStatusRequest, VotingSession,
};
use crate::schema::{candidates, settings, voting_sessions};

// Admin route to control voting
#[post("/admin/status", format = "json", data = "<status_request>")]
pub async fn set_voting_status(
    mut db: Connection<VotingDB>,
    status_request: Json<UpdateVotingStatusRequest>,
) -> Result<Status, Status> {
    // In a real app, add authentication here!

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

    Ok(Status::Ok)
}

// Admin route to get stats
#[get("/admin/stats")]
pub async fn get_stats(mut db: Connection<VotingDB>) -> Result<Json<i64>, Status> {
    use crate::schema::votes::dsl::votes;

    let count: i64 = votes.count().get_result(&mut db).await.map_err(|e| {
        eprintln!("Error getting stats: {}", e);
        Status::InternalServerError
    })?;

    Ok(Json(count))
}

// Route to get voting results
#[get("/admin/results")]
pub async fn get_results(
    mut db: Connection<VotingDB>,
) -> Result<Json<Vec<CandidateResult>>, Status> {
    let all_candidates = candidates::table
        .load::<Candidate>(&mut db)
        .await
        .map_err(|e| {
            eprintln!("Error loading candidates: {}", e);
            Status::InternalServerError
        })?;

    let mut results = Vec::new();

    for candidate in all_candidates {
        use crate::schema::votes::dsl::{candidate_id, votes};

        let count: i64 = votes
            .filter(candidate_id.eq(candidate.id))
            .count()
            .get_result(&mut db)
            .await
            .unwrap_or(0);

        results.push(CandidateResult {
            name: candidate.name,
            votes: count,
        });
    }

    Ok(Json(results))
}

// Route to pick a lottery winner
#[get("/admin/lottery")]
pub async fn pick_winner(mut db: Connection<VotingDB>) -> Result<Json<LotteryWinner>, Status> {
    // Get all sessions (potential winners)
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
