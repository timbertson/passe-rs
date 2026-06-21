use passe_core::auth::Authentication;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use anyhow::*;
use passe_core::password;
use passe_core::password::{Password, Domain};

use web_sys::{Request, RequestInit, RequestMode, Response};
use passe_core::config;

// #[wasm_bindgen]
// pub fn hello() -> String {
// 	println!("Hello from rust!");
// 	format!("helloo?")
// }

// #[wasm_bindgen]
// pub fn generate_password(domain: String, password: String) -> String {
// 	generate(Domain(&domain), Password(&password), DomainConfig::default())
// }

/*

Want to:

 - expose pointer to config
 - have load state and some minimal data exposed
 - perform queries & serialize state

Most of this can be opaque rust struct.

Box can work, but how do we allocate it globally?


*/

// pub struct JResult<T>(Result<T>);

// impl<T> From<Result<T>> for JResult<T> {
// 	fn from(result: Result<T>) -> Self {
// 		Self(result)
// 	}
// }

// pub struct JsonRef<T>(*mut T);
// impl<T> JsonRef<T> {
// 	pub fn into_pointer(value: T) -> Self {
// 		let boxed = Box::new(value);
// 		JsonRef(Box::into_raw(boxed))
// 	}
	
// 	pub fn access_mut<F, R>(ptr: *mut T, f: F) -> Result<R> where F: FnOnce(&mut T) -> Result<R> {
// 		let r: &mut T = unsafe { &mut *ptr };
// 		f(r)
// 	}

// 	// pub fn access<F, R>(ptr: *const Arc<T>, f: F) -> Result<R> where F: FnOnce(&mut T) -> Result<R> {
// 	// 	let arc: &Arc<T> = unsafe { &*ptr };
// 	// 	let r = arc.borrow();
// 	// 	f(r)
// 	// }

// 	pub fn consume_pointer(ptr: *mut Arc<T>) -> Arc<T> {
// 		unsafe { *Box::from_raw(ptr) }
// 	}
// }

// #[derive(Serialize)]
// pub struct ResultJson<T> {
// 	error: Option<String>,
// 	value: Option<T>,
// }

// impl<T> Into<JsValue> for JResult<T> where T: serde::Serialize + Debug {
// 	fn into(self) -> wasm_bindgen::JsValue {
// 		let struct_ = match self.0 {
// 			Result::Ok(v) => ResultJson { error: None, value: Some(v) },
// 			Err(e) => ResultJson { error: Some(format!("{:?}", e)), value: None }
// 		};
// 		serde_wasm_bindgen::to_value(&struct_).expect("Unserializable result value: {:?}")
// 	}
// }

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
	
	pub fn generate_password(&self, domain: String, password: String) -> String {
		let domain_config = self.0.for_domain(&domain);
		password::generate(Domain(&domain), Password(&password), domain_config.underlying())
	}
	
	pub async fn authenticate_request(&self) -> JsResult<Option<Request>> {
		if let Some(ref auth) = self.0.data.authentication {
			Ok(Some(authenticate_request(auth)?))
		} else {
			Ok(None)
		}
	}
}

const CONTENT_TYPE: &str = "content-type";
const JSON_TYPE: &str = "application/json";

pub fn authenticate_request(auth: &Authentication) -> JsResult<Request> {
	let opts = RequestInit::new();
	opts.set_method("POST");
	let request = Request::new_with_str_and_init("/authenticate", &opts)?;

	request.headers().set(CONTENT_TYPE, JSON_TYPE)?;
	request.headers().set("Authorization", &serde_json::to_string(auth).expect("Unserializable JSON"))?;
	Result::Ok(request)
}
