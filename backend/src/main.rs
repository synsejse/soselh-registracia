#[macro_use]
extern crate rocket;
use rocket::form::Form;
use rocket::fs::{FileServer, NamedFile};
use rocket::response::Redirect;
use serde::Deserialize;

#[derive(Debug, Deserialize, FromForm)]
#[allow(dead_code)]
struct ContactMessage {
    company: Option<String>, // Anti-bot field
    name: String,
    email: String,
    phone: Option<String>,
    subject: Option<String>,
    message: String,
}

#[post("/contact/message", data = "<form>")]
#[allow(dead_code)]
fn contact_message(form: Form<ContactMessage>) -> Redirect {
    let data = form.into_inner();

    // Check anti-bot field
    if data.company.is_some() && !data.company.as_ref().unwrap().is_empty() {
        println!("⚠️  Potential bot detected (company field filled)");
        return Redirect::to("/");
    }

    println!("📧 New contact message received:");
    println!("  Name: {}", data.name);
    println!("  Email: {}", data.email);
    if let Some(phone) = &data.phone.as_ref().filter(|p| !p.is_empty()) {
        println!("  Phone: {}", phone);
    }
    if let Some(subject) = &data.subject.as_ref().filter(|s| !s.is_empty()) {
        println!("  Subject: {}", subject);
    }
    println!("  Message: {}", data.message);
    println!("---");

    Redirect::to("/")
}

#[catch(404)]
async fn not_found() -> Option<NamedFile> {
    NamedFile::open("/app/static/404.html").await.ok()
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![contact_message])
        .mount("/", FileServer::from("/app/static"))
        .register("/", catchers![not_found])
}
