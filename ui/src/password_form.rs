use seed::{prelude::*, *};

#[derive(Debug, Clone)]
pub struct GeneratedPassword {
	password: String,
	visible: bool,
	fully_selected: bool,
}

pub struct DomainSuggestions {
	contents: Vec<String>,
	selected: Option<usize>,
}

pub struct State {
	domain: String,
	// db: Store.t
	master_password: String,
	generated_password: Option<GeneratedPassword>,
	// active_input: Option<InputTaget>,
	domain_suggestions: Option<DomainSuggestions>,
}

pub enum Msg {
}

fn view(state: &State) -> Node<Msg> {
	let domain_input = input![
		attrs!{ At::Value => state.domain },
	];

	div![
		C!["bg-slate-100"],
		domain_input,
	]
}
