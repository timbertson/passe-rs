use log::*;
use anyhow::*;
use serde::{Serialize, Deserialize};
use std::{fs, collections::BTreeMap, path::PathBuf};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigFile {
	#[serde(default)]
	credentials: Option<StoredCredentials>,

	#[serde(default)]
	defaults: DomainConfig,

	#[serde(default)]
	domains: BTreeMap<String, DomainConfig>,

	#[serde(default)]
	changes: BTreeMap<String, Change<DomainConfig>>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredCredentials {
	user: String,
	contents: String,
}

pub struct Config {
	stored: ConfigFile,
	unsaved_changes: Option<BTreeMap<String, Change<DomainConfig>>>
}

impl Default for ConfigFile {
	fn default() -> Self {
		Self {
			credentials: Default::default(),
			defaults: Default::default(),
			domains: Default::default(),
			changes: Default::default(),
		}
	}
}

impl Config {
	fn user_path() -> PathBuf {
		PathBuf::from(shellexpand::tilde("~/.config/passe/user.json").into_owned())
	}

	pub fn load_user() -> Result<Config> {
		let path = Self::user_path();
		let stored = if path.exists() {
			let contents = fs::read_to_string(&path)?;
			let stored = serde_json::from_str(&contents)
				.with_context(|| anyhow!("Processing {:?}", &path))?;
			stored
		} else {
			ConfigFile::default()
		};
		Ok(Self { stored, unsaved_changes: None })
	}

	pub fn save_user(&mut self) -> Result<()> {
		if let Some(changes) = &self.unsaved_changes {
			let path = Self::user_path();
			info!("Storing {}", &path.to_string_lossy());
			self.stored.changes = changes.to_owned();
			fs::write(&Self::user_path(), serde_json::to_string_pretty(&self.stored)?)?;
			self.unsaved_changes = None;
		}
		Ok(())
	}
	
	pub fn domain_list(&self) -> impl Iterator<Item=&String> + '_ {
		self.stored.domains.keys().into_iter()
	}

	fn override_for(&self, domain: &str) -> Option<&Change<DomainConfig>> {
		let unsaved: Option<&Change<DomainConfig>> = self.unsaved_changes.as_ref().and_then(|m| m.get(domain));
		unsaved.or_else(||self.stored.changes.get(domain))
	}

	pub fn for_domain(&self, domain: &str) -> Defaulted<&DomainConfig> {
		let stored = self.stored.domains.get(domain);
		let found = match self.override_for(domain) {
			Some(Change::Delete) => None,
			Some(Change::Set(ch)) => Some(ch),
			None => stored,
		};
		match found {
			Some(f) => Defaulted::Explicit(f),
			None => Defaulted::Default(&self.stored.defaults)
		}
	}

	pub fn add(&mut self, domain: String, domain_config: DomainConfig) {
		let unsaved = self.unsaved_changes.get_or_insert_with(Default::default);
		unsaved.insert(domain, Change::Set(domain_config));
	}
}
