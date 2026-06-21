use passe_core::auth::{Authentication, LoginRequest};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use anyhow::{Result};
use passe_core::password;
use passe_core::password::{Password, Domain};

use web_sys::{Request, RequestInit};
use passe_core::config::{self, ConfigFile};

const CONTENT_TYPE: &str = "content-type";
const AUTHORIZATION: &str = "authorization";
const JSON_TYPE: &str = "application/json";


type JsResult<T> = std::result::Result<T, JsValue>;

fn js<T>(result: Result<T>) -> JsResult<T> {
	match result {
		Result::Ok(e) => Result::Ok(e),
		Err(e) => Err(format!("{:?}", &e).into())
	}
}

#[wasm_bindgen]
pub struct Config(config::Config);

#[wasm_bindgen]
impl Config {
	#[wasm_bindgen]
	pub fn new(serialized_state: Option<String>) -> JsResult<Config> {
		println!("{:?}", &serialized_state);
		let config = match serialized_state {
			Some(s) => js(config::Config::deserialize(&s))?,
			None => Default::default(),
		};
		Result::Ok(Config(config))
	}
	
	pub fn serialize(&self) -> JsResult<String> {
		Ok(js(self.0.serialize())?)
	}

	pub fn update_after_save(&mut self) {
		self.0.update_after_save()
	}
	
	pub fn generate_password(&self, domain: String, password: String) -> String {
		let domain_config = self.0.for_domain(&domain);
		password::generate(Domain(&domain), Password(&password), domain_config.underlying())
	}
	
	pub fn authenticate_request(&self) -> JsResult<Option<Request>> {
		if let Some(ref auth) = self.0.data.authentication {
			Ok(Some(authenticate_request(auth)?))
		} else {
			Ok(None)
		}
	}

	pub fn login_request(&self, user: String, password: String) -> JsResult<Request> {
		Self::credential_request("/login", user, password)
	}

	pub fn register_request(&self, user: String, password: String) -> JsResult<Request> {
		Self::credential_request("/register", user, password)
	}
	
	fn credential_request(url: &'static str, user: String, password: String) -> JsResult<Request> {
		let data = LoginRequest { user, password };
		let opts = RequestInit::new();
		opts.set_method("POST");
		opts.set_body(&JsValue::from_str(&serde_json::to_string(&data).expect("Unserializable JSON")));
		let request = Request::new_with_str_and_init(url, &opts)?;
		request.headers().set(CONTENT_TYPE, JSON_TYPE)?;
		Result::Ok(request)
	}
	
	pub fn update_auth(&mut self, auth_json: JsValue) -> JsResult<()> {
		let auth_result = serde_wasm_bindgen::from_value(auth_json);
		self.0.update_after_login(auth_result?);
		Ok(())
	}

	pub fn sync_request(&self) -> JsResult<Request> {
		let auth = js(self.0.authentication())?;
		let data = &self.0.data;
		let opts = RequestInit::new();
		opts.set_method("POST");
		opts.set_body(&JsValue::from_str(&serde_json::to_string(&data).expect("Unserializable JSON")));
		let request = Request::new_with_str_and_init("/db", &opts)?;

		request.headers().set(CONTENT_TYPE, JSON_TYPE)?;
		request.headers().set(AUTHORIZATION, &serde_json::to_string(auth).expect("Unserializable JSON"))?;
		Result::Ok(request)
	}

	pub fn set_db(&mut self, db_json: JsValue) -> JsResult<()> {
		let mut db: ConfigFile = serde_wasm_bindgen::from_value(db_json)?;
		db.authentication = self.0.data.authentication.clone();
		self.0.data = db;
		Ok(())
	}
}

pub fn authenticate_request(auth: &Authentication) -> JsResult<Request> {
	let opts = RequestInit::new();
	opts.set_method("POST");
	let request = Request::new_with_str_and_init("/authenticate", &opts)?;

	request.headers().set(CONTENT_TYPE, JSON_TYPE)?;
	request.headers().set(AUTHORIZATION, &serde_json::to_string(auth).expect("Unserializable JSON"))?;
	Result::Ok(request)
}
