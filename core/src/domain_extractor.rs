use crate::domain_list;
use regex::Regex;

pub struct DomainExtractor(Regex);
impl Default for DomainExtractor {
	fn default() -> Self {
		Self(Regex::new(r"^(?:[^:]*://)?([^/]+)").unwrap())
	}
}

impl DomainExtractor {
	pub fn extract<'a, 'b>(&'a self, value: &'b str) -> &'b str {
		let host = self.host(value);

		let mut it = DotIterator::empty(host);
		it.expand(); // com
		it.expand(); // mydomain.com
		if domain_list::SECOND_LEVEL_DOMAINS.iter().any(|sld| *sld == it.result) {
			it.expand(); // mydomain.co.uk
		}
		it.result
	}
	
	pub fn host<'a,'b>(&'a self, maybe_url: &'b str) -> &'b str {
		// println!("RE: {:?} in {}", self.0.find(maybe_url), maybe_url);

		self.0.captures(maybe_url)
			.and_then(|c| c.get(1).map(|m| m.as_str())
		).unwrap_or(maybe_url)
	}
}

struct DotIterator<'a> {
	value: &'a str,
	
	len: usize,

	// a leading substring of `value`, i.e. [0...n]
	// Starts as the full string, shrinks over time
	prefix: &'a str,

	// a trailing substring of `value`, i.e. [n...(len-1)]
	// starts empty, expands over time
	pub result: &'a str,
}

const EMPTY_STR: &'static str = "";

impl<'a> DotIterator<'a> {
	pub fn empty(value: &'a str) -> Self {
		DotIterator {
			value,
			prefix: value,
			len: value.len(),
			result: EMPTY_STR,
		}
	}
	
	#[allow(deprecated)]
	pub fn expand(&mut self) {
		if let Some(idx) = self.prefix.rfind('.') {
			unsafe {
				self.prefix = self.value.slice_unchecked(0, idx);
				self.result = self.value.slice_unchecked(idx+1, self.len);
			}
			// println!("Expanded to index {}. prefix[{}], result[{}]", idx, self.prefix, self.result);
		} else {
			self.result = self.value;
		}
	}

	#[cfg(test)]
	fn expanded(mut self) -> Self {
		self.expand();
		self
	}
}

#[cfg(test)]
pub mod test {
use super::*;
	fn empty(s: &str) -> DotIterator<'_> {
		// println!("Start: {}", s);
		DotIterator::empty(s)
	}
	
	#[test]
	fn test_url_extractor() {
		let ex: DomainExtractor = Default::default();
		assert_eq!("my.com", ex.host("http://my.com"));
		assert_eq!("my.com", ex.host("://my.com"));
		assert_eq!("my.com", ex.host("ftp://my.com/foo/bar?yeah!"));
		assert_eq!("my.com", ex.host("http://my.com/foo/bar?redir=http://another.com/"));
		assert_eq!("my.com", ex.host("my.com/foo/bar?yeah!"));
	}

	#[test]
	fn test_dot_iterator() {
		// basic usage
		assert_eq!("com", empty("my.com").expanded().result);
		assert_eq!("my.com", empty("my.com").expanded().expanded().result);
		assert_eq!("my.com", empty("my.com").expanded().expanded().expanded().result);
		assert_eq!(".my.com", empty(".my.com").expanded().expanded().expanded().result);

		// dot edge cases
		assert_eq!("", empty("foo").result);
		assert_eq!("foo", empty("foo").expanded().result);
		assert_eq!("", empty("foo.").expanded().result);
		assert_eq!("foo.", empty("foo.").expanded().expanded().result);
		assert_eq!(".", empty(".....").expanded().expanded().result);

		// non-ASCII
		assert_eq!("🔥", empty("✨.🔥").expanded().result);
		assert_eq!("✨.🔥", empty("✨.🔥").expanded().expanded().result);
	}
}
