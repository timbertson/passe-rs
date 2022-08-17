use clipboard::{ClipboardProvider, ClipboardContext};
use log::*;
use anyhow::*;
use clap::{App, Arg};

use passe::*;
use passe::{password::*, config::{DomainConfig, Config}};

pub fn main() -> Result<()> {
	let app = App::new("passe")
		.arg(Arg::with_name("add").long("add"))
		.arg(Arg::with_name("sync").long("sync"))
		.arg(Arg::with_name("list").long("list").short('l'))
		.arg(Arg::with_name("domain"))
	;

	let opts = app.get_matches();
	debug!("cli opts: {:?}", &opts);
	
	let mut config = Config::load_user()?;
	let get_domain = || opts.get_one::<String>("domain").ok_or_else(|| anyhow!("Domain required"));

	if opts.contains_id("add") {
		let domain = get_domain()?;
		let mut domain_config = config.for_domain(&domain).underlying().to_owned();
		let note_str = rprompt::prompt_reply_stderr("Note: ")?;
		domain_config.note = if note_str.is_empty() {
			None
		} else {
			Some(note_str)
		};
		config.add(domain.to_owned(), domain_config);

	} else if opts.contains_id("list") {
		for domain in config.domain_list() {
			println!("{}", domain)
		}
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
	if opts.contains_id("sync") {
		println!("sync!");
	}

	Ok(())
}
