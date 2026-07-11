use std::{path::PathBuf, fs};
use std::env;

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
		let base_str = env::var("PASSE_SERVER_ROOT")
			.unwrap_or_else(|_| shellexpand::tilde("~/.config/passe-server").into_owned());
		let mut base = PathBuf::from(base_str);
		match file {
			File::LoginDB => base.push("users.json"),
			File::UserDB(u) => base.push(format!("user-{}.json", u)),
		}
		base
	}
	
	fn tmp_path(path: &PathBuf) -> PathBuf {
		let filename = path.file_name().map(|p| p.to_str().expect("non-utf8 filename")).unwrap_or_else(|| "");
		path.with_file_name(format!("{}.tmp", filename))
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
		let dest = FsPersistence::path(file);
		debug!("Saving file {:?}", &dest);
		let tmp_path = FsPersistence::tmp_path(&dest);
		fs::write(&tmp_path, contents)?;
		fs::rename(tmp_path, dest)?;
		Ok(())
	}
}
