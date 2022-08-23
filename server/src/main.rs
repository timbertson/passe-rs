#[macro_use] extern crate rocket;

mod error;
mod auth;
mod storage;
mod request;

use rocket::State;
use rocket::response;
use rocket::serde::json::Json;
use rocket::fs::FileServer;

use passe::auth::{LoginRequest, Authentication};

use crate::error::HttpResult;
use crate::request::*;

use anyhow::*;

#[get("/")]
fn index() -> response::Redirect {
	// TODO internal redirect instead of exposing URL?
	response::Redirect::to("/static/index.html")
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

#[post("/authenticate")]
fn authenticate(_user: AuthenticatedUser) -> HttpResult<()> {
	Result::Ok(())
}

#[get("/db")]
fn get_db(user: AuthenticatedUser, state: &State<DbMutex>) -> HttpResult<Json<passe::config::ConfigFile>> {
	Result::Ok(Json(state.lock().user_db(&user)?))
}

#[post("/db", data="<data>")]
fn post_db(data: String, user: AuthenticatedUser, state: &State<DbMutex>) -> HttpResult<Json<passe::config::ConfigFile>> {
	Result::Ok(Json(state.lock().user_db(&user)?))
}

#[launch]
fn rocket() -> _ {
	let config = rocket::Config {
		log_level: rocket::log::LogLevel::Normal,
		..rocket::Config::release_default()
	};
	rocket::custom(config)
		.manage(DbMutex::new().unwrap())
		.mount("/", routes![
			index,
			register,
			login,
			authenticate,
			get_db,
			post_db
		])
		.mount("/static", FileServer::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/static")))
		.mount("/pkg", FileServer::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/pkg")))
}
