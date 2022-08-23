mod password_form;

use seed::{prelude::*, *};

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
	Model { counter: 0 }
}

struct Model {
	counter: i32,
}

#[derive(Copy, Clone)]
enum Msg {
	Increment,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
	match msg {
		Msg::Increment => model.counter += 1,
	}
}

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
	div![
		C!["bg-slate-100"],
		div![
			"This is a counter: ",
			C!["counter"],
			button![model.counter, ev(Ev::Click, |_| Msg::Increment),],
		]
	]
}

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
	// Mount the `app` to the element with the `id` "app".
	App::start("app", init, update, view);
}
