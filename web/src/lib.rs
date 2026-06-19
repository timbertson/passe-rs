
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello() -> String {
	println!("Hello from rust!");
	format!("helloo?")
}
