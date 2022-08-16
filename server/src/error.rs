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

pub type HttpResult<T> = Result<T, HttpError>;

impl<'r, 'o: 'r> Responder<'r, 'o> for HttpError {
	fn respond_to(self, _: &'r Request) -> response::Result<'o> {
		Response::build()
			.status(http::Status::InternalServerError)
			.header(http::ContentType::JSON)
			.ok()
	}
}
