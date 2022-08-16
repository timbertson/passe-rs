use std::{path::PathBuf, fs};

use anyhow::*;

#[derive(Copy, Clone, Debug)]
pub enum File<'a> {
	LoginDB,
	UserDB(&'a str),
}

pub trait Persistence: std::fmt::Debug + Send + Sync + 'static {
	fn load(&self, file: File<'_>) -> Result<Option<String>>;

	fn save(&self, file: File<'_>, contents: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct FsPersistence;
impl FsPersistence {
	fn path(file: File<'_>) -> PathBuf {
		let mut base = PathBuf::from(shellexpand::tilde("~/.config/passe-server").into_owned());
		match file {
			File::LoginDB => base.push("users.json"),
			File::UserDB(u) => base.push(format!("user-{}.json", u)),
		}
		base
	}
}

impl Persistence for FsPersistence {
	fn load(&self, file: File<'_>) -> Result<Option<String>> {
		let path = FsPersistence::path(file);
		Ok(if path.exists() {
			Some(fs::read_to_string(path)?)
		} else {
			None
		})
	}

	fn save(&self, file: File<'_>, contents: &str) -> Result<()> {
		Ok(fs::write(FsPersistence::path(file), contents)?)
	}
}
