use chrono::NaiveDateTime;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;

use crate::schema::{candidates, settings, votes, voting_sessions};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = voting_sessions)]
pub struct VotingSession {
    pub session_token: String,
    pub display_name: String,
    pub ip_address: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub voter_id: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = voting_sessions)]
pub struct NewVotingSession {
    pub session_token: String,
    pub display_name: String,
    pub ip_address: Option<String>,
    pub voter_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = candidates)]
pub struct Candidate {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = candidates)]
pub struct NewCandidate {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = votes)]
pub struct Vote {
    pub id: i32,
    pub session_token: String,
    pub candidate_id: i32,
    pub voted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = votes)]
pub struct NewVote {
    pub session_token: String,
    pub candidate_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub key_name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateSessionRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CastVoteRequest {
    pub candidate_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateVotingStatusRequest {
    pub action: String,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CandidateResult {
    pub name: String,
    pub votes: i64,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LotteryWinner {
    pub name: String,
    pub voter_id: String,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct VotingStatusResponse {
    pub ready: bool,
    pub has_voted: bool,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SessionInfoResponse {
    pub voter_id: String,
    pub name: String,
}
