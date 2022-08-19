use log::*;
use anyhow::*;
use serde::{Serialize, Deserialize};
use std::{fs, collections::BTreeMap, path::PathBuf, ops::Deref};

use crate::auth::Authentication;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DomainConfig {
	pub length: usize,
	pub suffix: Option<String>, // suffix for password
	// pub post_suffix: Option<String>, // suffix for generated password
	pub note: Option<String>,
}

impl DomainConfig {
	pub fn print(&self) {
		if let Some(suffix) = &self.suffix {
			println!("Suffix: {}", suffix);
		}

		if let Some(note) = &self.note {
			println!("Note: {}", note);
		}
	}
}

impl Default for DomainConfig {
	fn default() -> Self {
		Self {
			length: 10,
			suffix: Default::default(),
			note: Default::default()
		}
	}
}

impl DomainConfig {
	#![allow(dead_code)]
	pub fn with_length(self, length: usize) -> Self {
		Self {
			length,
			suffix: self.suffix,
			note: self.note,
		}
	}
}

type Changes = BTreeMap<String, Change<DomainConfig>>;
type Domains = BTreeMap<String, DomainConfig>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigFile {
	#[serde(default)]
	pub authentication: Option<Authentication>,

	#[serde(default)]
	defaults: DomainConfig,

	#[serde(default)]
	domains: Domains,

	#[serde(default)]
	changes: Changes,
}

impl Default for ConfigFile {
	fn default() -> Self {
		Self {
			authentication: Default::default(),
			defaults: Default::default(),
			domains: Default::default(),
			changes: Default::default(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Change<T> {
	Delete,
	Set(T),
}

#[derive(Debug, Clone)]
pub enum Defaulted<T> {
	Explicit(T),
	Default(T),
}

impl<T> AsRef<T> for Defaulted<T> {
	fn as_ref(&self) -> &T {
		match &self {
			Defaulted::Explicit(t) => t,
			Defaulted::Default(t) => t,
		}
	}
}

impl<T> Defaulted<T> {
	pub fn underlying(self) -> T {
		match self {
			Defaulted::Explicit(t) => t,
			Defaulted::Default(t) => t,
		}
	}
}

pub struct Config {
	pub data: ConfigFile,
	pub dirty: bool,
}

impl Deref for Config {
	type Target = ConfigFile;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl Config {
	fn user_path() -> PathBuf {
		PathBuf::from(shellexpand::tilde("~/.config/passe/user.json").into_owned())
	}

	pub fn load_user() -> Result<Config> {
		let path = Self::user_path();
		let data = if path.exists() {
			let contents = fs::read_to_string(&path)?;
			serde_json::from_str::<ConfigFile>(&contents)
				.with_context(|| anyhow!("Processing {:?}", &path))?
		} else {
			ConfigFile::default()
		};
		Ok(Self { data, dirty: false })
	}

	pub fn save_user(&mut self) -> Result<()> {
		if self.dirty {
			let path = Self::user_path();
			info!("Storing {}", &path.to_string_lossy());
			fs::write(&Self::user_path(), serde_json::to_string_pretty(&self.data)?)?;
			self.dirty = false;
		}
		Ok(())
	}
	
	pub fn changes(&self) -> &Changes {
		&self.data.changes
	}
	
	pub fn domain_list(&self) -> impl Iterator<Item=&String> + '_ {
		self.domains.keys().into_iter()
	}

	fn override_for(&self, domain: &str) -> Option<&Change<DomainConfig>> {
		self.changes.get(domain)
	}
	
	pub fn post_sync(&mut self, merged: Domains) {
		self.data.domains = merged;
		self.data.changes = Default::default();
	}

	pub fn for_domain(&self, domain: &str) -> Defaulted<&DomainConfig> {
		let stored = self.domains.get(domain);
		let found = match self.override_for(domain) {
			Some(Change::Delete) => None,
			Some(Change::Set(ch)) => Some(ch),
			None => stored,
		};
		match found {
			Some(f) => Defaulted::Explicit(f),
			None => Defaulted::Default(&self.defaults)
		}
	}

	pub fn add(&mut self, domain: String, domain_config: DomainConfig) {
		self.data.changes.insert(domain, Change::Set(domain_config));
		self.dirty = true;
	}
}
