// Routes module - organizes all HTTP route handlers

pub mod voting;

use rocket::fs::NamedFile;
use rocket::http::Status;

/// 404 error handler - serves custom 404.html page
#[catch(404)]
pub async fn not_found() -> Option<NamedFile> {
    NamedFile::open("/app/static/404.html").await.ok()
}

#[catch(401)]
pub fn unauthorized() -> Status {
    Status::Unauthorized
}
