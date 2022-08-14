use log::*;
use base64::engine::fast_portable::{FastPortable, self};

use crate::config::DomainConfig;

#[derive(Clone, Debug, Copy)]
pub struct Domain<'a>(pub &'a str);

pub struct Password<'a>(pub &'a str);

struct Gen {
	value: Vec<u8>,
	buf: String,
	engine: FastPortable,
	// r64_config: CustomConfig,
	length: usize,
}

impl Gen {
	fn new(value: String, config: &DomainConfig) -> Self {

		// NOTE: base64 requires lossless encoding.
		// Hoever SGP reuses 9 & 8, plus A for padding. See `substitute` function below
		let alpha = base64::alphabet::Alphabet::from_str(
			"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789./").unwrap();
		let engine = fast_portable::FastPortable::from(&alpha, fast_portable::PAD);

		Self {
			value: value.into_bytes(),
			buf: String::new(),
			engine,
			length: config.length,
		}
	}

	fn substitute(byte: u8) -> u8 {
		match byte {
			ch if ch == '=' as u8 => 'A' as u8,
			ch if ch == '.' as u8 => '9' as u8,
			ch if ch == '/' as u8 => '8' as u8,
			ch => ch,
		}
	}

	fn chars(&self) -> impl Iterator<Item=char> + '_{
		self.value.iter().take(self.length).map(|u| *u as char)
	}
	
	fn valid(&self) -> bool {
		let leading_lower = self.chars().take(1).any(|ch| ch.is_ascii_lowercase());
		let has_upper = self.chars().any(|ch| ch.is_ascii_uppercase());
		let has_digit = self.chars().any(|ch| ch.is_ascii_digit());
		leading_lower && has_upper && has_digit
	}
	
	fn iterate(&mut self) {
		self.buf.clear();
		let digest = md5::compute(&self.value);
		base64::encode_engine_string(digest.0, &mut self.buf, &self.engine);
		self.value.clear();
		for b in self.buf.bytes() {
			self.value.push(Self::substitute(b))
		}
		debug!(" -> {}", String::from_utf8(self.value.clone()).unwrap());
	}
	
	fn run(mut self) -> String {
		let mut i = 0;
		while i < 10 {
			i += 1;
			self.iterate();
		}

		while !self.valid() {
			i += 1;
			debug!("Iteration: {}", i);
			if i > 50 {
				panic!("Didn't find a valid password after {} iterations", i);
			}
			self.iterate();
		}
		
		self.chars().collect()
	}
}

pub fn generate(domain: Domain, password: Password, config: DomainConfig) -> String {
	let input = format!("{}{}:{}", password.0, config.suffix.as_deref().unwrap_or(""), domain.0);
	Gen::new(input, &config).run()
}


#[cfg(test)]
pub mod test {
	use super::*;

	#[test]
	pub fn test_sample() {
		let generated = generate(
			Domain("example.org"),
			Password("secret"),
			DomainConfig::default().with_length(10)
		);
		assert_eq!(generated, "tYb1lyMQLA");
	}
}
