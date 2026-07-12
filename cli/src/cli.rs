use std::borrow::Cow;

use log::*;
use anyhow::*;
use clap::{Arg, ArgAction, Command};
use arboard::Clipboard;
use ureq::Agent;
use ureq::tls::{TlsConfig, RootCerts};

use passe_core::*;
use passe_core::password::*;
use passe_core::config::{Config, Domains};
use passe_core::auth::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn main() -> Result<()> {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
	let app = Command::new("passe")
		.arg(Arg::new("edit").long("edit").action(ArgAction::SetTrue))
		.arg(Arg::new("sync").long("sync").action(ArgAction::SetTrue))
		.arg(Arg::new("full").long("full").action(ArgAction::SetTrue).help("Do a full (initial) sync"))
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

		let agent = Agent::config_builder()
			.tls_config(TlsConfig::builder().root_certs(RootCerts::PlatformVerifier).build())
			.build()
			.new_agent();

		let changes = if opts.contains_id("full") {
			config.full_changes()
		} else {
			config.changes().to_owned()
		};
		let sync_result: Domains = authed_request(&agent, &mut config, "db", Some(&changes))?;
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
			println!("* new domain: {}", &domain);
		}
		let password = rpassword::prompt_password("Password: ").unwrap();
		let generated = password::generate(Domain(domain), Password(&password), domain_config.underlying());

		// finalize early in this branch, since we wait below and an impatient user may ctrl+c
		finalize(&mut config)?;

		match Clipboard::new() {
			Result::Ok(mut clipboard) => {
				let suffix = "";

				#[cfg(target_os = "linux")]
				let set = clipboard.set().wait();
				#[cfg(target_os = "linux")]
				let suffix = "; paste to terminate";

				#[cfg(not(target_os = "linux"))]
				let set = clipboard.set();

				println!("(copied to your clipboard{})", suffix);
				set.text(&generated)?;
			},
			Result::Err(e) => {
				error!("Clipboard failed: {:?}", e);
				rpassword::prompt_password("Press return to print password ...").unwrap();
				println!("{}", &generated);
			}
		}
	}

	finalize(&mut config)
}

fn finalize(config: &mut Config) -> Result<()> {
	config.save_user()
}

fn make_url(suffix: &'static str) -> String {
	let root = std::env::var("PASSE_SERVER").unwrap_or_else(|_| "https://passe-458142165195.australia-southeast2.run.app".to_owned());
	format!("{}/{}", root, suffix)
}

fn login(agent: &Agent, credentials: &LoginRequest) -> Result<Authentication> {
	Ok(agent.post(&make_url("login"))
		.send_json(&credentials)?
		.body_mut()
		.read_json::<Authentication>()?)
}

fn authed_request<Data: Serialize, Response: DeserializeOwned>(agent: &Agent, auth_manager: &mut dyn AuthManager, path: &'static str, data: Option<&Data>) -> Result<Response> {
	let url = make_url(path);
	debug!("Request URL: {}", &url);
	let do_req = |auth: &Authentication| {
		let auth_header = serde_json::to_string(auth)?;
		let response = match data {
			None => agent.get(&url)
				.header("Authorization", &auth_header)
				.call(),

			Some(data) => agent.post(&url)
				.header("Authorization", &auth_header)
				.header("Content-type", "application/json")
				.send_json(data),
		};
		match response {
			Result::Ok(mut result) => Ok(Some(result.body_mut().read_json::<Response>()?)),
			Result::Err(ureq::Error::StatusCode(401)) => {
				debug!("401; returning None");
				Ok(None)
			},
			Result::Err(other) => {
				debug!("Unknown error: {:?}", &other);
				Err(other.into())
			},
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
			let auth = login(&agent, &creds)?;
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
