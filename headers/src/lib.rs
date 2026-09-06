use request::http_error::HttpParseError;
use std::collections::HashMap;

pub struct Headers {
	field_lines: HashMap<String, Vec<String>>,
}

impl Headers {
	pub fn new() -> Headers {
		Headers {
			field_lines: HashMap::new(),
		}
	}

	pub fn get(&self, header: &str) -> String {
		let field_values = &self.field_lines[header];
		let mut value = String::new();

		if field_values.len() > 1 {
			for i in 0..field_values.len() {
				if i == field_values.len() - 1 {
					value.push_str(field_values[i].as_str());
					break;
				}

				let val = field_values[i].clone() + ", ";
				value.push_str(val.as_str());
			}
		} else {
			value = field_values[0].clone();
		}

		value
	}

	pub fn set(&mut self, header: String, value: String) -> Result<(), HttpParseError> {
		let cl = "content-length".to_string();
		let host = "host".to_string();

		if header.to_lowercase() == cl {
			if !self.field_lines.contains_key(&cl) {
				self.field_lines.insert(cl, vec![value]);
				return Ok(());
			}

			let values = match self.field_lines.get(cl.as_str()) {
				Some(v) => v,
				None => return Err(HttpParseError::EmptyFieldValue),
			};

			if values[0] != value {
				return Err(HttpParseError::InvalidDuplicateHeader);
			}

			return Ok(());
		}

		if header.to_lowercase() == host {
			if !self.field_lines.contains_key(&host) {
				self.field_lines.insert(host, vec![value]);
				return Ok(());
			}

			return Err(HttpParseError::InvalidDuplicateHeader);
		}

		self
			.field_lines
			.entry(header.to_lowercase())
			.and_modify(|v| v.push(value.clone()))
			.or_insert(vec![value]);

		Ok(())
	}

	pub fn parse_headers(&mut self, data: &[u8]) -> Result<(usize, bool), HttpParseError> {
		// TODO: technically must be within the US-ASCII subset for safety
		let buf = match String::from_utf8(data.to_vec()) {
			Ok(s) => s,
			Err(_) => return Err(HttpParseError::InvalidHeaderChars),
		};

		if !buf.contains("\r\n") {
			return Ok((0, false));
		}

		let mut done = false;

		let parts: Vec<&str> = buf.split_inclusive("\r\n").collect();

		if parts[0] == "\r\n" {
			done = true;
			return Ok((parts[0].len(), done));
		}

		dbg!(parts[0].len());

		let (name, value) = match parts[0].split_once(" ") {
			Some(line) => line,
			None => return Err(HttpParseError::HeaderParseError),
		};

		let value = value.trim();

		let parsed: usize = name.len() + value.len() + 3; // add one space and "\r\n" lengths

		if !name.contains(":") {
			return Err(HttpParseError::InvalidHeaderWhitespace);
		}

		let delim = match name.find(":") {
			Some(d) => d,
			None => return Err(HttpParseError::NoColonInHeader),
		};

		let name = &name[..delim];

		if !Self::is_valid_header(name.to_string()) {
			return Err(HttpParseError::InvalidHeaderChars);
		}

		let _ = match self.set(name.to_string(), value.to_string()) {
			Ok(_) => (),
			Err(err) => return Err(err),
		};

		Ok((parsed, done))
	}

	fn is_valid_header(header: String) -> bool {
		header.chars().all(|c| {
			c.is_alphanumeric()
				|| matches!(
					c,
					'!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '.' | '^' | '_' | '`' | '|' | '~'
				)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn valid_single_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host: localhost:42069\r\n\r\n";

		let (parsed, done) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done);
		assert_eq!(headers.get("host"), "localhost:42069");
		assert_eq!(parsed, 23);
	}

	#[test]
	fn valid_single_extra_whitespace_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host:       localhost:42069\r\n\r\n";

		let (parsed, done) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done);
		assert_eq!(headers.get("host"), "localhost:42069");
		assert_eq!(parsed, 23);
	}

	#[test]
	fn valid_two_headers_with_existing() {
		let mut headers = Headers::new();
		headers.field_lines = HashMap::from([
			("Host".to_string(), vec!["localhost:42069".to_string()]),
			("Content-Length:".to_string(), vec!["45".to_string()]),
		]);
		let data: &[u8] = b"Accept: application/json\r\nCache-Control: no-cache\r\n\r\n";

		let (parsed1, done1) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		let (parsed2, done2) = match headers.parse_headers(&data[parsed1..]) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done1);
		assert!(!done2);

		assert_eq!(headers.get("accept"), "application/json");
		assert_eq!(parsed1, 26);
		assert_eq!(headers.get("cache-control"), "no-cache");
		assert_eq!(parsed2, 25);
	}

	#[test]
	fn valid_done_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"\r\n";

		let (parsed, done) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(done);
		assert_eq!(parsed, 2);
	}

	#[test]
	fn valid_duplicate_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Set-Cookie: beep=boop\r\nSet-Cookie: meep=moop\r\n\r\n";

		let (parsed1, done1) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		let (parsed2, done2) = match headers.parse_headers(&data[parsed1..]) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done1);
		assert!(!done2);

		assert_eq!(parsed1, 23);
		assert_eq!(parsed2, 23);

		assert_eq!(
			headers.get("set-cookie"),
			"beep=boop, meep=moop".to_string()
		);
	}

	#[test]
	fn valid_content_length_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Content-Length: 23\r\nContent-Length: 23\r\n\r\n";

		let (parsed1, done1) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		let (parsed2, done2) = match headers.parse_headers(&data[parsed1..]) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done1);
		assert!(!done2);

		assert_eq!(parsed1, 20);
		assert_eq!(parsed2, 20);

		assert_eq!(headers.get("content-length"), "23");
	}

	#[test]
	fn invalid_leading_whitespace_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"       Host: localhost:42069\r\n\r\n";

		let _ = match headers.parse_headers(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidHeaderWhitespace, err),
		};
	}

	#[test]
	fn invalid_whitespace_in_field_name() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host : localhost:42069\r\n\r\n";

		let _ = match headers.parse_headers(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidHeaderWhitespace, err),
		};
	}

	#[test]
	fn invalid_char_in_field_name() {
		let mut headers = Headers::new();
		let data: &[u8] = b"H\xA9st: localhost:42069\r\n\r\n";

		let _ = match headers.parse_headers(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidHeaderChars, err),
		};
	}

	#[test]
	fn invalid_duplicate_same_host_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host: example.com\r\nHost: example.com\r\n\r\n";

		let (parsed, done) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done);
		assert_eq!(parsed, 19);

		let _ = match headers.parse_headers(&data[parsed..]) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}

	#[test]
	fn invalid_duplicate_different_host_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host: example.com\r\nHost: example.com\r\n\r\n";

		let (parsed, done) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done);
		assert_eq!(parsed, 19);

		let _ = match headers.parse_headers(&data[parsed..]) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}

	#[test]
	fn invalid_duplicate_content_length_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Content-Length: 23\r\nContent-Length: 45\r\n";

		let (parsed, done) = match headers.parse_headers(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(!done);
		assert_eq!(parsed, 20);

		let _ = match headers.parse_headers(&data[parsed..]) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}
}
