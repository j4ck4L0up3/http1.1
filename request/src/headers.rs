use crate::http_error::HttpParseError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Headers {
	pub field_lines: HashMap<Box<str>, Vec<Box<str>>>,
}

impl Headers {
	pub fn new() -> Headers {
		Headers {
			field_lines: HashMap::new(),
		}
	}

	pub fn get(&self, header: &str) -> String {
		let mut value = String::new();
		let field_values = match self
			.field_lines
			.get(&header.to_lowercase().into_boxed_str())
		{
			Some(vals) => vals,
			None => return value,
		};

		if field_values.len() > 1 {
			for i in 0..field_values.len() {
				if i == field_values.len() - 1 {
					value.push_str(&field_values[i]);
					break;
				}

				let val = String::from(&*field_values[i]) + ", ";
				value.push_str(val.as_str());
			}
		} else {
			value = String::from(&*field_values[0]);
		}

		value
	}

	pub fn set(&mut self, name: String, value: String) -> Result<(), HttpParseError> {
		let cl = "content-length".to_string().into_boxed_str();
		let host = "host".to_string().into_boxed_str();

		if name.to_lowercase() == *cl {
			if !self.field_lines.contains_key(&cl) {
				self.field_lines.insert(cl, vec![value.into_boxed_str()]);
				return Ok(());
			}

			let values = match self.field_lines.get(&cl) {
				Some(v) => v,
				None => return Err(HttpParseError::EmptyFieldValue),
			};

			if *values[0] != value {
				return Err(HttpParseError::InvalidDuplicateHeader);
			}

			return Ok(());
		}

		if name.to_lowercase() == *host {
			if !self.field_lines.contains_key(&host) {
				self.field_lines.insert(host, vec![value.into_boxed_str()]);
				return Ok(());
			}

			return Err(HttpParseError::InvalidDuplicateHeader);
		}

		self
			.field_lines
			.entry(name.to_lowercase().into_boxed_str())
			.and_modify(|v| v.push(value.clone().into_boxed_str()))
			.or_insert(vec![value.into_boxed_str()]);

		Ok(())
	}

	pub fn parse(&mut self, data: &[u8]) -> Result<(usize, bool), HttpParseError> {
		let crlf = "\r\n";
		let idx: usize = data.len();
		let mut parsed: usize = 0;
		let mut done = false;

		loop {
			// find crlf and return its inclusive position
			let idx: usize = match &data[parsed..idx]
				.windows(crlf.len())
				.position(|s| s == crlf.as_bytes())
				.map(|pos| pos + crlf.len())
			{
				Some(i) => *i,
				None => break,
			};

			if idx == 0 {
				done = true;
				break;
			}

			let p = match self.parse_header(&data[parsed..idx + parsed]) {
				Ok(p) => p,
				Err(err) => return Err(err),
			};
			parsed += p;

			if p == crlf.len() {
				done = true;
				break;
			}

			if parsed == data.len() {
				break;
			}
		}

		Ok((parsed, done))
	}

	fn parse_header(&mut self, data: &[u8]) -> Result<usize, HttpParseError> {
		let mut parsed: usize = 0;
		let crlf = "\r\n";

		let buf = match String::from_utf8(data.to_vec()) {
			Ok(s) => s,
			Err(_) => return Err(HttpParseError::InvalidHeaderChars),
		};

		if !buf.is_ascii() {
			return Err(HttpParseError::InvalidASCII);
		}

		if buf == crlf {
			return Ok(crlf.len());
		}

		let (name, value) = match buf.split_once(':') {
			Some((n, v)) => (n.to_string(), v.to_string()),
			None => return Err(HttpParseError::NoColonInHeader),
		};

		let value = value.trim();
		if value == "" {
			return Err(HttpParseError::EmptyFieldValue);
		}

		if name.chars().any(|c| c.is_whitespace()) {
			return Err(HttpParseError::InvalidHeaderWhitespace);
		}

		if !Self::is_valid_header(&name) {
			return Err(HttpParseError::InvalidHeaderChars);
		}

		if let Err(err) = self.set(name, value.to_string()) {
			return Err(err);
		}

		parsed += buf.len();

		Ok(parsed)
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

		let (parsed, done) = match headers.parse(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(done);
		assert_eq!(headers.get("host"), "localhost:42069");
		assert_eq!(parsed, 25);
	}

	#[test]
	fn valid_single_extra_whitespace_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host:       localhost:42069\r\n\r\n";

		let (parsed, done) = match headers.parse(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(done);
		assert_eq!(headers.get("host"), "localhost:42069");
		assert_eq!(parsed, 31);
	}

	#[test]
	fn valid_two_headers_with_existing() {
		let mut headers = Headers::new();

		let name1: Box<str> = "Host".into();
		let value1: Box<str> = "localhost:42069".into();
		let name2: Box<str> = "Content-Length:".into();
		let value2: Box<str> = "45".into();

		headers.field_lines = HashMap::from([(name1, vec![value1]), (name2, vec![value2])]);
		let data: &[u8] = b"Accept: application/json\r\nCache-Control: no-cache\r\n\r\n";

		let (parsed, done) = match headers.parse(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(done);
		assert_eq!(headers.get("accept"), "application/json");
		assert_eq!(headers.get("cache-control"), "no-cache");
		assert_eq!(parsed, 53);
	}

	#[test]
	fn valid_done_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"\r\n";

		let (parsed, done) = match headers.parse(data) {
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

		let (parsed, done) = match headers.parse(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(done);
		assert_eq!(parsed, 48);
		assert_eq!(
			headers.get("set-cookie"),
			"beep=boop, meep=moop".to_string()
		);
	}

	#[test]
	fn valid_content_length_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Content-Length: 23\r\nContent-Length: 23\r\n\r\n";

		let (parsed, done) = match headers.parse(data) {
			Ok(tup) => tup,
			Err(err) => panic!("expected parsed bytes, received error: {err}"),
		};

		assert!(done);
		assert_eq!(parsed, 42);
		assert_eq!(headers.get("content-length"), "23");
	}

	#[test]
	fn invalid_leading_whitespace_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"       Host: localhost:42069\r\n\r\n";

		let _ = match headers.parse(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidHeaderWhitespace, err),
		};
	}

	#[test]
	fn invalid_whitespace_in_field_name() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host : localhost:42069\r\n\r\n";

		let _ = match headers.parse(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidHeaderWhitespace, err),
		};
	}

	#[test]
	fn invalid_char_in_field_name() {
		let mut headers = Headers::new();
		let data: &[u8] = b"H\xA9st: localhost:42069\r\n\r\n";

		let _ = match headers.parse(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidHeaderChars, err),
		};
	}

	#[test]
	fn invalid_duplicate_same_host_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host: example.com\r\nHost: example.com\r\n\r\n";

		let _ = match headers.parse(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}

	#[test]
	fn invalid_duplicate_different_host_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Host: example.com\r\nHost: example.com\r\n\r\n";

		let _ = match headers.parse(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}

	#[test]
	fn invalid_duplicate_content_length_header() {
		let mut headers = Headers::new();
		let data: &[u8] = b"Content-Length: 23\r\nContent-Length: 45\r\n";

		let _ = match headers.parse(data) {
			Ok(tup) => panic!("expected error, received: {tup:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}
}
