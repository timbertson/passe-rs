use std::borrow::Cow;

use clipboard::{ClipboardProvider, ClipboardContext};
use log::*;
use anyhow::*;
use clap::{App, Arg};

use passe::*;
use passe::password::*;
use passe::config::{DomainConfig, Config};
use passe::auth::*;
use serde::de::DeserializeOwned;

pub fn main() -> Result<()> {
	let app = App::new("passe")
		.arg(Arg::with_name("edit").long("edit"))
		.arg(Arg::with_name("sync").long("sync"))
		.arg(Arg::with_name("full").long("full").help("Do a full (initial) sync"))
		.arg(Arg::with_name("list").long("list").short('l'))
		.arg(Arg::with_name("domain"))
	;

	let opts = app.get_matches();
	debug!("cli opts: {:?}", &opts);
	
	let mut config = Config::load_user()?;
	let get_domain = || opts.get_one::<String>("domain").ok_or_else(|| anyhow!("Domain required"));

	if opts.contains_id("list") {
		for domain in config.domain_list() {
			println!("{}", domain)
		}

	} else if opts.contains_id("edit") {
		let domain = get_domain().context("for --edit")?;
		let mut domain_config = config.for_domain(&domain).underlying().to_owned();
		edit_setting("Note", &mut domain_config.note)?;
		edit_setting("Suffix", &mut domain_config.suffix)?;

	} else if opts.contains_id("sync") {
		info!("Syncing ...");
		let changes = if opts.contains_id("full") {
			Cow::Owned(config.full_changes())
		} else {
			Cow::Borrowed(config.changes())
		};
		let data = serde_json::to_string(&changes)?;
		let sync_result = authed_request(&mut config, "db", Some(&data))?;
		config.post_sync(sync_result);

	} else {
		let domain = get_domain()?;
		println!("Domain: {}", domain);
		let domain_config = config.for_domain(&domain);
		debug!("domain config: {:?}", &domain_config);
		domain_config.as_ref().print();
		if let config::Defaulted::Default(_) = domain_config {
			warn!("** This is a new domain**");
		}
		let password = rpassword::prompt_password("Master password: ").unwrap();
		let gen = || password::generate(Domain(domain), Password(&password), DomainConfig {
			suffix: None,
			note: None,
			length: 10,
		});

		let copied: Result<(), Box<dyn std::error::Error>> = clipboard::ClipboardProvider::new()
			.and_then(|mut clip: ClipboardContext| clip.set_contents(gen()));
		match copied {
			Result::Ok(()) => {
				println!("(copied to your clipboard)");
			},
			Result::Err(e) => {
				error!("Clipboard failed: {:?}", e);
				rpassword::prompt_password("Press return to print password ...").unwrap();
				println!("{}", gen());
			}
		}
	}

	config.save_user()?;
	Ok(())
}

fn make_url(suffix: &'static str) -> String {
	let root = std::env::var("PASSE_SERVER").unwrap_or_else(|_| "http://localhost:8000".to_owned());
	format!("{}/{}", root, suffix)
}

fn login(credentials: &LoginRequest) -> Result<Authentication> {
	let response = ureq::post(&make_url("authenticate"))
		.send_string(&serde_json::to_string(credentials)?)?;
	Ok(response.into_json::<Authentication>()?)
}

fn authed_request<T: DeserializeOwned>(auth_manager: &mut dyn AuthManager, path: &'static str, data: Option<&str>) -> Result<T> {
	let url = make_url(path);
	let do_req = |auth: &Authentication| {
		let auth_header = serde_json::to_string(auth)?;
		let response = match data {
			None => ureq::get(&url)
				.set("Authorization", &auth_header)
				.call(),

			Some(data) => ureq::post(&url)
				.set("Authorization", &auth_header)
				.set("Content-type", "application/json")
				.send_string(data),
		};
		match response {
			Result::Ok(result) => Ok(Some(result.into_json::<T>()?)),
			Result::Err(ureq::Error::Status(401, _)) => Ok(None),
			Result::Err(other) => Err(other.into()),
		}
	};

	match auth_manager.get().and_then(|auth| match do_req(auth) {
		Result::Ok(None) => None,
		Result::Ok(Some(t)) => Some(Ok(t)),
		Result::Err(err) => Some(Err(err)),
	}) {
		Some(result) => result,
		None => {
			let creds = auth_manager.ask_credentials()?;
			let auth = login(&creds)?;
			auth_manager.set(auth.clone());
			do_req(&auth)?.ok_or_else(||anyhow!("Unauthorized"))
		}
	}
}

trait AuthManager {
	fn get(&self) -> Option<&Authentication>;
	fn ask_credentials(&self) -> Result<LoginRequest>;
	fn set(&mut self, auth: Authentication) -> ();
}

impl AuthManager for Config {
	fn get(&self) -> Option<&Authentication> {
		self.data.authentication.as_ref()
	}

	fn ask_credentials(&self) -> Result<LoginRequest> {
		let existing = self.get().map(|auth| &auth.user);
		let prompt_suffix = match existing {
			Some(u) => Cow::Owned(format!("[{}]", u)),
			None => Cow::Borrowed(""),
		};
		let mut user = rprompt::prompt_reply_stderr(&format!("User: {}", prompt_suffix))?;
		if user.is_empty() {
			user = existing.ok_or_else(||anyhow!("user required"))?.to_owned();
		}
		let password = rpassword::prompt_password("Sync password: ")?;
		Ok(LoginRequest { user, password })
	}

	fn set(&mut self, auth: Authentication) -> () {
		self.data.authentication = Some(auth);
		self.dirty = true;
	}
}


fn edit_setting(desc: &str, config: &mut Option<String>) -> Result<()> {
	let prompt = match config {
		Some(s) => format!("{}: [{}] ", desc, s),
		None => format!("{}: ", desc),
	};
	let response = rprompt::prompt_reply_stderr(&prompt)?;
	if !response.is_empty() {
		*config = Some(response);
	}
	Ok(())
}
