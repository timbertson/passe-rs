use serde::*;
use serde::ser::SerializeStruct;
use rocket::response::{Response, Responder};
use rocket::response;
use rocket::http;
use rocket::request::Request;

use anyhow::*;


pub struct HttpError(anyhow::Error);
impl From<anyhow::Error> for HttpError {
	fn from(err: anyhow::Error) -> Self {
		HttpError(err)
	}
}
impl Serialize for HttpError {
	fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
		let mut state = s.serialize_struct("HttpError", 1)?;
		state.serialize_field("message", &format!("{:?}", &self.0))?;
		state.end()
	}
}

pub type HttpResult<T> = Result<T, HttpError>;

impl<'r, 'o: 'r> Responder<'r, 'o> for HttpError {
	fn respond_to(self, _: &'r Request) -> response::Result<'o> {
		let json_str = serde_json::to_string(&self).expect("error serialization failed");
		Response::build()
			.status(http::Status::InternalServerError)
			.header(http::ContentType::JSON)
			.sized_body(None, std::io::Cursor::new(json_str))
			.ok()
	}
}
