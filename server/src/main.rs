#[macro_use] extern crate rocket;

mod error;
mod auth;

use std::sync::{Mutex, MutexGuard};
use std::{path::PathBuf, ops::Deref};
use rocket::State;
use rocket::serde::json::Json;

use auth::{UserDB, LoginRequest};
use crate::error::{HttpResult};

use anyhow::*;


struct DbMutex(Mutex<UserDB>);
impl DbMutex {
	fn lock(&self) -> MutexGuard<'_, UserDB> {
		self.0.lock().unwrap()
	}
}

#[get("/")]
fn index(state: &State<UserDB>) -> String {
	format!("Hello, world! {:?}", state.deref())
}

#[post("/login", data="<data>")]
fn login(data: Json<LoginRequest>, state: &State<DbMutex>) -> HttpResult<String> {
	state.lock().login(data.deref())?;
	Result::Ok(format!("Hello, world!"))
}

fn init_state() -> Result<DbMutex> {
	let path = PathBuf::from(shellexpand::tilde("~/.config/passe-server/users.json").into_owned());
	Ok(DbMutex(Mutex::new(if path.exists() {
		let contents = std::fs::read_to_string(&path)?;
		UserDB::load(&contents)?
	} else {
		UserDB::default()
	})))
}

#[launch]
fn rocket() -> _ {
	rocket::build()
		.manage(init_state().unwrap())
		.mount("/", routes![index])
}
