use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
	pub user: String,
	pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authentication {
	pub user: String,
	pub token: String,
}
