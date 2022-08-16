#[macro_use] extern crate rocket;

mod error;
mod auth;
mod storage;

use std::sync::{Mutex, MutexGuard};
use std::{path::PathBuf, ops::Deref};
use rocket::State;
use rocket::serde::json::Json;

use auth::{UserDB, LoginRequest, Authentication, Token};
use serde::{Serialize, Deserialize};
use storage::Persistence;
use crate::error::{HttpResult};

use anyhow::*;

struct DbMutex(Mutex<UserDB>);
impl DbMutex {
	fn lock(&self) -> MutexGuard<'_, UserDB> {
		self.0.lock().unwrap()
	}
}

#[get("/")]
fn index() -> String {
	format!("Hello, world!")
}

#[post("/register", data="<data>")]
fn register(data: Json<LoginRequest>, state: &State<DbMutex>) -> HttpResult<Json<Authentication>> {
	state.lock().register(&data)?;
	login(data, state)
}

#[post("/login", data="<data>")]
fn login(data: Json<LoginRequest>, state: &State<DbMutex>) -> HttpResult<Json<Authentication>> {
	let login_request = data.0;
	let token = state.lock().login(&login_request)?;
	Result::Ok(Json(Authentication {
		user: login_request.user, token: token.value
	}))
}

#[post("/authenticate", data="<data>")]
fn authenticate(data: Json<Authentication>, state: &State<DbMutex>) -> HttpResult<()> {
	state.lock().validate(data.deref())?;
	Result::Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PostDB {
	authentication: Authentication,
	contents: String, // TODO
}

#[get("/db", data="<data>")]
fn get_db(data: Json<Authentication>, state: &State<DbMutex>) -> HttpResult<&'static str> {
	state.lock().validate(data.deref())?;
	Result::Ok("TODO")
}

#[post("/db", data="<data>")]
fn post_db(data: Json<PostDB>, state: &State<DbMutex>) -> HttpResult<&'static str> {
	state.lock().validate(&data.authentication)?;
	Result::Ok("TODO")
}

fn init_state() -> Result<DbMutex> {
	Ok(DbMutex(Mutex::new(UserDB::new(storage::FsPersistence)?)))
}

#[launch]
fn rocket() -> _ {
	rocket::build()
		.manage(init_state().unwrap())
		.mount("/", routes![
			index,
			register,
			login,
			authenticate,
			get_db,
			post_db
		])
}
