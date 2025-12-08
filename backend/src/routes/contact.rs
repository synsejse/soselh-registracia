// Contact form submission route handler

// use rocket::form::Form;
// use rocket::response::Redirect;
// use rocket_db_pools::Connection;
// use rocket_db_pools::diesel::prelude::*;

// use crate::db::MessagesDB;
// use crate::models::{ContactMessage, ContactMessageForm};
// use crate::schema::messages;

// /// Handle contact form submission
// #[post("/contact/message", data = "<form>")]
// pub async fn submit_message(
//     mut db: Connection<MessagesDB>,
//     form: Form<ContactMessageForm>,
// ) -> Redirect {
//     let data = form.into_inner();

//     // Check honeypot field to detect bots
//     if data.is_bot() {
//         println!("⚠️  Bot detected, ignoring submission");
//         return Redirect::to("/");
//     }

//     // TODO: Add input validation here

//     // Insert message into database
//     let result = db
//         .transaction(|mut conn| {
//             Box::pin(async move {
//                 diesel::insert_into(messages::table)
//                     .values(ContactMessage::from(data))
//                     .execute(&mut conn)
//                     .await?;

//                 Ok::<_, diesel::result::Error>(())
//             })
//         })
//         .await;

//     if let Err(e) = result {
//         eprintln!("❌ Failed to save contact message: {}", e);
//     }

//     Redirect::to("/")
// }
