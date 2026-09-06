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

		if !Self::is_valid_header(&name.to_string()) {
			return Err(HttpParseError::InvalidHeaderChars);
		}

		self
			.field_lines
			.insert(name.to_string().to_lowercase(), vec![value.to_string()]);

		Ok((parsed, done))
	}

	fn is_valid_header(header: &String) -> bool {
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

		assert!(parsed > 0);
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

		assert!(parsed > 0);
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

		assert!(parsed1 > 0);
		assert!(!done1);

		assert!(parsed2 > 0);
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

		assert!(parsed == 2);
		assert!(done);
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
}
