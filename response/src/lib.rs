mod status_codes;

use request::{headers::Headers, http_error::HttpParseError};
use status_codes::StatusCode;

pub struct Response {
	pub status_code: StatusCode,
	pub headers: Headers,
}

impl Default for Response {
	fn default() -> Self {
		Response {
			status_code: StatusCode::OK,
			headers: Self::get_default_headers(0),
		}
	}
}

impl Response {
	fn get_default_headers(content_len: usize) -> Headers {
		let mut headers = Headers::new();

		let result = headers.set(String::from("Content-Length"), content_len.to_string());
		assert!(result.is_ok());

		let result = headers.set(String::from("Content-Type"), String::from("text/plain"));
		assert!(result.is_ok());

		let result = headers.set(String::from("Connection"), String::from("close"));
		assert!(result.is_ok());

		headers
	}

	pub fn set_header(&mut self, name: &str, value: &str) -> Result<(), HttpParseError> {
		self.headers.set(String::from(name), String::from(value))
	}

	pub fn as_bytes(&self) -> Vec<u8> {
		let mut response = String::new();

		response.push_str(format!("{}", self.status_code).as_str());
		response.push('\r');
		response.push('\n');

		for key in self.headers.field_lines.keys() {
			response.push_str(key);
			response.push(':');
			response.push(' ');
			response.push_str(self.headers.get(key).as_str());
			response.push('\r');
			response.push('\n');
		}

		response.into_bytes()
	}
}
