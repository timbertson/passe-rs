
use std::sync::{Mutex, MutexGuard};
use rocket::State;
use rocket::request::Outcome;
use rocket::http;

use passe::auth::*;
use crate::storage;
use crate::auth::UserDB;

use anyhow::*;

pub struct DbMutex(Mutex<UserDB>);
impl DbMutex {
	pub fn lock(&self) -> MutexGuard<'_, UserDB> {
		self.0.lock().unwrap()
	}

	pub fn new() -> Result<DbMutex> {
		Ok(DbMutex(Mutex::new(UserDB::new(storage::FsPersistence)?)))
	}
}

pub struct AuthenticatedUser(String);

impl AuthenticatedUser {
	pub fn name(&self) -> &str {
		&self.0
	}
}

#[async_trait]
impl<'r> rocket::request::FromRequest<'r> for AuthenticatedUser {
	type Error = &'static str;

	async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
		// TODO lots of pattern matching here, seems primitive
		match request.headers().get_one("authorization") {
			None => Outcome::Failure((http::Status::BadRequest, "header missing")),
			Some(header) => {
				match serde_json::from_str::<Authentication>(header) {
					Result::Ok(auth) => {
						request.guard::<&State<DbMutex>>().await
						.map_failure(|_| (http::Status::InternalServerError, "state missing"))
						.and_then(|db| {
							match db.lock().validate(&auth) {
								Result::Ok(()) => Outcome::Success(AuthenticatedUser(auth.user)),
								Result::Err(_) => Outcome::Failure((http::Status::Unauthorized, "validation failed")),
							}
						})
					},
					Result::Err(_) => Outcome::Failure((http::Status::BadRequest, "parsing failed")),
				}
			}
		}
	}
}
