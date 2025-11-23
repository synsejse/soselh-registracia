// Data models for contact messages

use rocket::form::FromForm;
use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::diesel::prelude::*;

use crate::schema::messages;

/// Form data received from the contact form
#[derive(Debug, Clone, Deserialize, Serialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct ContactMessageForm {
    pub company: Option<String>, // Anti-bot honeypot field
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub subject: Option<String>,
    pub message: String,
}

/// Database representation of a contact message
#[derive(Insertable)]
#[diesel(table_name = messages)]
pub struct ContactMessage {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub subject: Option<String>,
    pub message: String,
}

impl From<ContactMessageForm> for ContactMessage {
    fn from(form: ContactMessageForm) -> Self {
        ContactMessage {
            id: None,
            name: form.name,
            email: form.email,
            phone: form.phone,
            subject: form.subject,
            message: form.message,
        }
    }
}

impl ContactMessageForm {
    /// Check if this submission is likely from a bot
    pub fn is_bot(&self) -> bool {
        self.company.as_ref().is_some_and(|c| !c.is_empty())
    }
}
