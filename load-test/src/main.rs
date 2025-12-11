use anyhow::{Context, Result};
use clap::Parser;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL (e.g., http://localhost:8080)
    #[arg(short, long, default_value = "http://localhost:8080")]
    url: String,

    /// Number of users to simulate
    #[arg(short, long, default_value_t = 100)]
    users: usize,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    /// Presenter password for admin actions
    #[arg(short, long, default_value = "password")]
    password: String,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    id: i32,
    // name: String,
}

#[derive(Serialize)]
struct CreateSessionRequest {
    name: String,
}

#[derive(Serialize)]
struct CastVoteRequest {
    candidate_id: i32,
}

#[derive(Serialize)]
struct AdminStatusRequest {
    action: String,
}

#[derive(Serialize)]
struct PresenterLoginRequest {
    password: String,
}

async fn run_user_simulation(
    client: &Client,
    base_url: &str,
    user_id: usize,
    candidates: &[Candidate],
) -> Result<()> {
    // 1. Create Session
    let session_url = format!("{}/api/session", base_url);
    let name = format!("LoadTestUser_{}", user_id);

    let _session_res = client
        .post(&session_url)
        .json(&CreateSessionRequest { name })
        .send()
        .await
        .context("Failed to send session request")?
        .error_for_status()
        .context("Session creation failed")?;

    // 2. Pick a candidate
    let mut rng = rand::thread_rng();
    let candidate = candidates
        .choose(&mut rng)
        .context("No candidates available")?;

    // 3. Vote
    let vote_url = format!("{}/api/vote", base_url);
    client
        .post(&vote_url)
        .json(&CastVoteRequest {
            candidate_id: candidate.id,
        })
        .send()
        .await
        .context("Failed to send vote request")?
        .error_for_status()
        .context("Vote casting failed")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🚀 Starting load test against {}", args.url);
    println!("👥 Users: {}", args.users);
    println!("⚡ Concurrency: {}", args.concurrency);

    // 0. Setup: Enable voting via admin API (just in case)
    let admin_client = Client::builder()
        .cookie_store(true)
        .build()
        .context("Failed to build admin client")?;

    // Login first
    let login_url = format!("{}/api/presenter/login", args.url);
    admin_client
        .post(&login_url)
        .json(&PresenterLoginRequest {
            password: args.password.clone(),
        })
        .send()
        .await
        .context("Failed to send login request")?
        .error_for_status()
        .context("Failed to login as presenter")?;

    println!("🔑 Logged in as presenter");

    let status_url = format!("{}/api/admin/status", args.url);
    admin_client
        .post(&status_url)
        .json(&AdminStatusRequest {
            action: "start".to_string(),
        })
        .send()
        .await
        .context("Failed to enable voting")?
        .error_for_status()
        .context("Failed to set voting status to start")?;

    println!("✅ Voting enabled via Admin API");

    // 1. Fetch candidates once
    let candidates_url = format!("{}/api/candidates", args.url);
    let candidates: Vec<Candidate> = admin_client
        .get(&candidates_url)
        .send()
        .await
        .context("Failed to fetch candidates")?
        .json()
        .await
        .context("Failed to parse candidates")?;

    if candidates.is_empty() {
        anyhow::bail!("No candidates found on the server. Cannot vote.");
    }
    println!("📋 Found {} candidates", candidates.len());

    let candidates = Arc::new(candidates);
    let base_url = Arc::new(args.url.clone());

    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let pb = ProgressBar::new(args.users as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let start_time = Instant::now();

    // Create a stream of futures
    let results = stream::iter(0..args.users)
        .map(|i| {
            let base_url = base_url.clone();
            let candidates = candidates.clone();
            let success_count = success_count.clone();
            let failure_count = failure_count.clone();
            let pb = pb.clone();

            async move {
                // Create a dedicated client for this user to isolate cookies
                let client = Client::builder().cookie_store(true).build().unwrap();

                match run_user_simulation(&client, &base_url, i, &candidates).await {
                    Ok(_) => {
                        success_count.fetch_add(1, Ordering::Relaxed);
                        pb.set_message(format!(
                            "Success: {}",
                            success_count.load(Ordering::Relaxed)
                        ));
                    }
                    Err(_e) => {
                        failure_count.fetch_add(1, Ordering::Relaxed);
                        pb.set_message(format!(
                            "Errors: {}",
                            failure_count.load(Ordering::Relaxed)
                        ));
                    }
                }
                pb.inc(1);
            }
        })
        .buffer_unordered(args.concurrency)
        .collect::<Vec<()>>();

    results.await;

    pb.finish_with_message("Done");

    let duration = start_time.elapsed();
    let successes = success_count.load(Ordering::Relaxed);
    let failures = failure_count.load(Ordering::Relaxed);
    let rps = successes as f64 / duration.as_secs_f64();

    println!("\n📊 Results:");
    println!("   Time taken: {:?}", duration);
    println!("   Total requests: {}", args.users);
    println!("   Successful votes: {}", successes);
    println!("   Failed votes: {}", failures);
    println!("   Throughput: {:.2} votes/sec", rps);

    Ok(())
}
