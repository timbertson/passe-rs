use log::*;
use anyhow::*;
use serde::{Serialize, Deserialize};
use std::{fs, collections::BTreeMap, path::PathBuf, ops::Deref};
use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::auth::Authentication;
use crate::domain_extractor::DomainExtractor;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

pub type Changes = BTreeMap<String, Change<DomainConfig>>;
pub type Domains = BTreeMap<String, DomainConfig>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigFile {
	#[serde(default)]
	pub authentication: Option<Authentication>,

	#[serde(default)]
	pub defaults: DomainConfig,

	#[serde(default)]
	pub domains: Domains,

	#[serde(default)]
	pub changes: Changes,
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

#[derive(Debug, PartialEq, Eq)]
struct LengthStr<'a> {
	value: &'a str,
	len: usize
}

impl<'a> LengthStr<'a> {
	fn new(value: &'a str) -> Self {
		LengthStr { value, len: value.len() }
	}
}

impl<'a> Ord for LengthStr<'a> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.len.cmp(&other.len)
	}
}

impl<'a> PartialOrd for LengthStr<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
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
	pub fn explicit(self) -> Option<T> {
		match self {
			Defaulted::Explicit(t) => Some(t),
			Defaulted::Default(_) => None,
		}
	}

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
	extractor: DomainExtractor,
}

impl Default for Config {
	fn default() -> Self {
		Self { data: Default::default(), dirty: false, extractor: Default::default() }
	}
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
	
	pub fn deserialize(s: &str) -> Result<Config> {
		let data = serde_json::from_str::<ConfigFile>(s)
			.context("Deserializing user config")?;
		Ok(Self { data, dirty: false, extractor: Default::default() })
	}

	pub fn load_user() -> Result<Config> {
		let path = Self::user_path();
		let result = if path.exists() {
			info!("Loading {:?}", &path);
			let contents = fs::read_to_string(&path)?;
			Self::deserialize(&contents).with_context(|| format!("Processing {:?}", &path))?
		} else {
			debug!("No config exists at {:?}", &path);
			Default::default()
		};
		Ok(result)
	}

	pub fn save_user(&mut self) -> Result<()> {
		if self.dirty {
			let path = Self::user_path();
			info!("Storing {}", &path.to_string_lossy());
			fs::write(&Self::user_path(), serde_json::to_string_pretty(&self.data)?)?;
			self.update_after_save();
		}
		Ok(())
	}
	
	pub fn authentication(&self) -> Result<&Authentication> {
		self.data.authentication.as_ref().ok_or_else(||anyhow!("Authorization missing"))
	}

	pub fn update_after_save(&mut self) {
		self.dirty = false;
	}

	pub fn serialize(&self) -> Result<String> {
		Ok(serde_json::to_string_pretty(&self.data)?)
	}
	
	pub fn changes(&self) -> &Changes {
		&self.data.changes
	}

	pub fn full_changes(&self) -> Changes {
		let mut result: Changes = self.data.domains.iter()
			.map(|(k,v)| (k.clone(), Change::Set(v.clone())))
			.collect();
		result.append(&mut self.data.changes.clone());
		result
	}
	
	pub fn domain_list(&self) -> impl Iterator<Item=&str> + '_ {
		let mut set = BTreeSet::from_iter(self.domains.keys());
		set.extend(self.changes().keys());
		set.into_iter().map(|s| s.as_ref())
	}

	pub fn domains_matching<'a, 'b>(&'a self, partial: &'b str, limit: usize) -> Vec<&'a str> {
		let sorted = BTreeSet::from_iter(self.domain_list().map(LengthStr::new).into_iter());
		sorted.into_iter()
			.filter(|candidate| candidate.value.contains(partial))
			.map(|length_str| length_str.value)
			.take(limit)
			.collect()
	}
	
	pub fn extract_domain<'a, 'b>(&'a self, value: &'b str) -> Option<&'b str> {
		let extracted = self.extractor.extract(value);
		if extracted == value {
			None
		} else {
			Some(extracted)
		}
	}

	fn override_for(&self, domain: &str) -> Option<&Change<DomainConfig>> {
		self.changes.get(domain)
	}
	
	pub fn update_after_sync(&mut self, merged: Domains) {
		self.data.domains = merged;
		self.data.changes = Default::default();
	}

	pub fn update_after_login(&mut self, auth: Authentication) {
		self.data.authentication = Some(auth);
		self.dirty = true;
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
		if let Some(existing) = self.for_domain(&domain).explicit() {
			if existing == &domain_config {
				info!("Skipping save for unchanged domain {}", &domain);
				return;
			}
		}

		info!("Updated domain {}", &domain);
		self.data.changes.insert(domain, Change::Set(domain_config));
		info!("Changes is now: {:?}", &self.data.changes);
		self.dirty = true;
	}
}
