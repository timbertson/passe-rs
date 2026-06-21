use std::borrow::Cow;

mod clipboard;

use log::*;
use anyhow::*;
use clap::{Arg, ArgAction, Command};

use passe_core::*;
use passe_core::password::*;
use passe_core::config::Config;
use passe_core::auth::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn main() -> Result<()> {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
	let app = Command::new("passe")
		.arg(Arg::new("edit").long("edit").action(ArgAction::SetTrue))
		.arg(Arg::new("sync").long("sync").action(ArgAction::SetTrue))
		// .arg(Arg::new("full").long("full").help("Do a full (initial) sync"))
		.arg(Arg::new("list").long("list").short('l').action(ArgAction::SetTrue))
		.arg(Arg::new("domain").required(false))
	;

	let opts = app.get_matches();
	debug!("cli opts: {:?}", &opts);
	
	let mut config = Config::load_user()?;
	let get_domain = || opts.get_one::<String>("domain").ok_or_else(|| anyhow!("Domain required"));

	if opts.get_flag("list") {
		for domain in config.domain_list() {
			println!("{}", domain)
		}
	} else if opts.get_flag("sync") {
		info!("Syncing ...");
		let changes = if opts.contains_id("full") {
			Cow::Owned(config.full_changes())
		} else {
			Cow::Borrowed(config.changes())
		};
		let data = serde_json::to_string(&changes)?;
		let sync_result = authed_request(&mut config, "db", Some(&data))?;
		config.update_after_sync(sync_result);
	} else if opts.get_flag("edit") {
		let domain = get_domain().context("for --edit")?;
		let mut domain_config = config.for_domain(&domain).underlying().to_owned();
		edit_setting("Note", &mut domain_config.note)?;
		edit_setting("Suffix", &mut domain_config.suffix)?;
		config.add(domain.to_owned(), domain_config);
	} else {
		let domain = get_domain()?;
		println!("Domain: {}", domain);
		let domain_config = config.for_domain(&domain);
		debug!("domain config: {:?}", &domain_config);
		domain_config.as_ref().print();
		if let config::Defaulted::Default(_) = domain_config {
			println!("* This is a new domain");
		}
		let password = rpassword::prompt_password("Master password: ").unwrap();
		let generated = password::generate(Domain(domain), Password(&password), domain_config.underlying());

		let copied: Result<()> = clipboard::copy(&generated);
		match copied {
			Result::Ok(()) => {
				println!("(copied to your clipboard)");
			},
			Result::Err(e) => {
				error!("Clipboard failed: {:?}", e);
				rpassword::prompt_password("Press return to print password ...").unwrap();
				println!("{}", &generated);
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
	Ok(ureq::post(&make_url("authenticate"))
		.send_json(&credentials)?
		.body_mut()
		.read_json::<Authentication>()?)
}

fn authed_request<Data: Serialize, Response: DeserializeOwned>(auth_manager: &mut dyn AuthManager, path: &'static str, data: Option<&Data>) -> Result<Response> {
	let url = make_url(path);
	let do_req = |auth: &Authentication| {
		let auth_header = serde_json::to_string(auth)?;
		let response = match data {
			None => ureq::get(&url)
				.header("Authorization", &auth_header)
				.call(),

			Some(data) => ureq::post(&url)
				.header("Authorization", &auth_header)
				.header("Content-type", "application/json")
				.send_json(data),
		};
		match response {
			Result::Ok(mut result) => Ok(Some(result.body_mut().read_json::<Response>()?)),
			Result::Err(ureq::Error::StatusCode(401)) => Ok(None),
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
		let mut user = rprompt::prompt_reply(&format!("User: {}", prompt_suffix))?;
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
	let response = rprompt::prompt_reply(&prompt)?;
	if !response.is_empty() {
		*config = Some(response);
	}
	Ok(())
}
