use std::{collections::HashMap, time::SystemTime};

use crate::storage::Persistence;
use crate::storage::File;
use rand::RngCore;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use anyhow::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PasswordConfig {
	iterations: u32,
	salt: Vec<u8>,
}

impl PasswordConfig {
	fn new() -> Self {
		let mut rng = rand::thread_rng();
		let mut salt: [u8; 16] = [0; 16];
		rng.try_fill_bytes(&mut salt).unwrap();
		Self { iterations: 10, salt: salt.into() }
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Password {
	value: Vec<u8>,
	config: PasswordConfig,
}

impl Password {
	pub fn new(password: &str) -> Result<Self> {
		let config = PasswordConfig::new();
		let value = Self::hash(password, &config)?;
		Ok(Password { config, value: value.into() })
	}

	fn hash(password: &str, config: &PasswordConfig) -> Result<[u8; 128]> {
		let mut output: [u8; 128] = [0; 128];
		bcrypt_pbkdf::bcrypt_pbkdf(password, &config.salt, config.iterations, &mut output)?;
		Ok(output)
	}

	pub fn validate(&self, password: &str) -> Result<bool> {
		Ok(self.value == Self::hash(password, &self.config)?)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EpochSeconds(u64);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Token {
	pub value: String,
	pub expires: EpochSeconds,
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

impl User {
	fn new(password: Password) -> Self {
		User { password, tokens: Vec::new() }
	}
}

#[derive(Deserialize)]
pub struct LoginRequest {
	pub user: String,
	pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authentication {
	pub user: String,
	pub token: String,
}

fn now() -> Result<EpochSeconds> {
	let sys = SystemTime::now();
	let secs = sys.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
	Ok(EpochSeconds(secs))
}

const EXPIRY_SECONDS: u64 = 60 * 60 * 24 * 7; // 1 week

impl User {
	pub fn login(&mut self, password: &str) -> Result<Token> {
		self.expire_tokens()?;
		if self.password.validate(password)? {
			let token = Token::new()?;
			self.tokens.push(token.clone());
			Ok(token)
		} else {
			Err(anyhow!("Unauthenticated"))
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

#[derive(Debug)]
pub struct UserDB {
	users: HashMap<String, User>,
	stored_users: HashMap<String, User>,
	persistence: Box<dyn Persistence>,
}

impl UserDB {
	fn load_file<T, P: Persistence + ?Sized>(persistence: &P, file: File) -> Result<T> where T: DeserializeOwned + Default {
		let contents = persistence.load(file)?;
		match contents {
			Some(contents) => Ok(serde_json::from_str(&contents)?),
			None => Ok(Default::default()),
		}
	}

	pub fn new<P: Persistence>(persistence: P) -> Result<UserDB> {
		let users: HashMap<String, User> = Self::load_file(&persistence, File::LoginDB)?;
		Ok(Self {
			users: users.clone(),
			stored_users: users,
			persistence: Box::new(persistence),
		})
	}
	
	pub fn register(&mut self, request: &LoginRequest) -> Result<()> {
		if self.users.contains_key(&request.user) {
			Err(anyhow!("Registration error"))
		} else {
			let password = Password::new(&request.password)?;
			self.users.insert(request.user.clone(), User::new(password));
			self.autosave()?;
			Ok(())
		}
	}
	
	pub fn login(&mut self, request: &LoginRequest) -> Result<Token> {
		let user = self.get_mut(&request.user)?;
		let token = user.login(&request.password)?;
		self.autosave()?;
		Ok(token)
	}

	pub fn validate(&mut self, request: &Authentication) -> Result<()> {
		let user = self.get_mut(&request.user)?;
		user.validate_token(&request.token)
	}
	
	pub fn get_mut(&mut self, username: &str) -> Result<&mut User> {
		self.users.get_mut(username).ok_or_else(||anyhow!("Unauthenticated!"))
	}

	fn is_dirty(&self) -> bool {
		self.stored_users != self.users
	}

	fn autosave(&mut self) -> Result<()> {
		if self.is_dirty() {
			Self::save_file(self.persistence.as_ref(), File::LoginDB, &self.users)?;
			self.stored_users = self.users.clone();
		}
		Ok(())
	}

	fn save_file<T, P: Persistence + ?Sized>(persistence: &P, file: File, t: &T) -> Result<()> where T: Serialize {
		info!("Saving {:?}", file);
		persistence.save(file, &serde_json::to_string(t)?)
	}

	pub fn serialize(&self) -> Result<String> {
		Ok(serde_json::to_string(&self.users)?)
	}
}
