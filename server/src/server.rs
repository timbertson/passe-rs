#[macro_use] extern crate rocket;

mod error;
mod db;
mod storage;
mod request;

use std::net::{IpAddr, Ipv4Addr};
use rocket::State;
use rocket::response;
use rocket::serde::json::Json;
use rocket::fs::{self, FileServer};

use passe_core::auth::{LoginRequest, Authentication};
use passe_core::config;

use crate::error::HttpResult;
use crate::request::*;

use anyhow::*;

#[get("/")]
fn index() -> response::Redirect {
	// TODO internal redirect instead of exposing URL?
	response::Redirect::to("/ui/public/index.html")
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
fn authenticate(user: AuthenticatedUser) -> HttpResult<Json<String>> {
	Result::Ok(Json(user.name().to_owned()))
}

#[get("/db")]
fn get_db(user: AuthenticatedUser, state: &State<DbMutex>) -> HttpResult<Json<config::ConfigFile>> {
	Result::Ok(Json(state.lock().user_db(&user)?))
}

#[post("/db", data="<data>")]
fn post_db(user: AuthenticatedUser, data: Json<config::Changes>, state: &State<DbMutex>) -> HttpResult<Json<config::ConfigFile>> {
	Result::Ok(Json(state.lock().sync_changes(&user, data.0)?))
}

#[launch]
fn rocket() -> _ {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
	let config = rocket::Config {
		log_level: rocket::log::LogLevel::Normal,
		address: IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)),
		port: 8080,
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
		// these mirror the on-disk layout for consistency, but don't expose anything outside the public folders
		.mount("/ui/public", FileServer::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/public"),
			fs::Options::None
		))
		.mount("/wasm/public", FileServer::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../wasm/public")))

		// also mount two special files at the root:
		// - serviceWorker.js, since it's limited to the path where it was loaded from
		// - index.html on /
		.mount("/serviceWorker.js", FileServer::new(
			concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/public/bundle/serviceWorker.js"),
			fs::Options::IndexFile
		).rank(1))
		.mount("/", FileServer::new(
			concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/public/index.html"),
			fs::Options::IndexFile
		).rank(2))
}
