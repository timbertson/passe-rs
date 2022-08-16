use std::{collections::HashMap, time::SystemTime};

use rand::RngCore;
use serde::{Serialize, Deserialize};
use anyhow::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Password {
	iterations: u32,
	salt: Vec<u8>,
	value: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EpochSeconds(u64);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Token {
	value: String,
	expires: EpochSeconds,
}

impl Token {
	fn new() -> Result<Token> {
		let mut rng = rand::thread_rng();
		let mut token_bytes: [u8; 64] = [0; 64];
		rng.try_fill_bytes(&mut token_bytes)?;
		let mut expires = now()?;
		expires.0 += EXPIRY_SECONDS;
		let value = base64::encode(token_bytes);
		Ok(Self { value, expires })
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
	password: Password,
	tokens: Vec<Token>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
	user: String,
	password: String,
}

#[derive(Deserialize)]
pub struct ValidateTokenRequest {
	user: String,
	token: String,
}

fn now() -> Result<EpochSeconds> {
	let sys = SystemTime::now();
	let secs = sys.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
	Ok(EpochSeconds(secs))
}

const EXPIRY_SECONDS: u64 = 60 * 60 * 24 * 7; // 1 week

impl User {
	pub fn login(&mut self, password: &str) -> Result<Token> {
		self.expire_tokens();
		let mut output: [u8; 128] = [0; 128];
		bcrypt_pbkdf::bcrypt_pbkdf(password, &self.password.salt, self.password.iterations, &mut output)?;
		if self.password.value != output {
			Err(anyhow!("Unauthenticated"))
		} else {
			let token = Token::new()?;
			self.tokens.push(token.clone());
			Ok(token)
		}
	}
	
	pub fn expire_tokens(&mut self) -> Result<()> {
		let min = now()?;
		self.tokens.retain_mut(|tok| tok.expires > min);
		Ok(())
	}

	pub fn validate_token(&mut self, token: &str) -> Result<()> {
		self.expire_tokens()?;
		if self.tokens.iter().any(|tok| tok.value == token) {
			Ok(())
		} else {
			Err(anyhow!("Unauthenticated"))
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct UserDB {
	users: HashMap<String, User>,
	stored_users: HashMap<String, User>,
}

impl UserDB {
	pub fn load(contents: &str) -> Result<UserDB> {
		let users: HashMap<String, User> = serde_json::from_str(contents)?;
		Ok(Self {
			users: users.clone(),
			stored_users: users,
		})
	}
	
	pub fn login(&mut self, request: &LoginRequest) -> Result<Token> {
		let user = self.get_mut(&request.user)?;
		user.login(&request.password)
	}

	pub fn validate(&mut self, request: &ValidateTokenRequest) -> Result<()> {
		let user = self.get_mut(&request.user)?;
		user.validate_token(&request.token)
	}
	
	pub fn get_mut(&mut self, username: &str) -> Result<&mut User> {
		self.users.get_mut(username).ok_or_else(||anyhow!("Unauthenticated!"))
	}

	pub fn is_dirty(&self) -> bool {
		self.stored_users != self.users
	}

	pub fn mark_saved(&mut self) {
		self.stored_users = self.users.clone()
	}

	pub fn serialize(&self) -> Result<String> {
		Ok(serde_json::to_string(&self.users)?)
	}
}
