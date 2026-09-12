use crate::method::Method;
use crate::HttpParseError;
use crate::HTTP_VERSION;
use std::fmt;

#[derive(PartialEq, Clone, Debug)]
pub struct RequestLine {
	pub method: Option<Method>,
	pub request_target: Box<str>,
	pub http_version: Box<str>,
}

impl RequestLine {
	pub fn parse(data: &[u8]) -> Result<(Option<RequestLine>, usize), HttpParseError> {
		let mut parsed: usize = 0;
		let crlf = "\r\n";

		let buf = match String::from_utf8(data.to_vec()) {
			Ok(b) => b,
			Err(_) => return Err(HttpParseError::RequestLineParseError),
		};

		if !buf.is_ascii() {
			return Err(HttpParseError::InvalidASCII);
		}

		if !buf.contains(crlf) {
			return Ok((None, 0));
		}

		let parts: Vec<&str> = buf.split(crlf).collect();
		let raw_req_line: Vec<&str> = parts[0].split(" ").collect();

		let method = match Method::parse(raw_req_line[0]) {
			Ok(m) => m,
			Err(err) => return Err(err),
		};

		let mut request_target = String::new();
		if raw_req_line[1].starts_with("/") {
			request_target.push_str(raw_req_line[1]);
		} else {
			return Err(HttpParseError::MissingRequestTarget);
		}
		let request_target = request_target.into_boxed_str();

		let version_number = match raw_req_line[2].strip_prefix("HTTP/") {
			Some(num) => num,
			None => return Err(HttpParseError::MissingHttpVersion),
		};

		let mut http_version = String::new();
		if version_number == HTTP_VERSION {
			http_version.push_str(HTTP_VERSION);
		} else {
			return Err(HttpParseError::WrongHttpVersion);
		}
		let http_version = http_version.into_boxed_str();

		let request_line = RequestLine {
			method: Some(method),
			request_target,
			http_version,
		};

		parsed += parts[0].len() + crlf.len();

		Ok((Some(request_line), parsed))
	}
}

impl fmt::Display for RequestLine {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Request Line:\n- Method: {}\n- Target: {}\n- Version: {}\n",
			self.method.as_ref().unwrap(),
			self.request_target,
			self.http_version,
		)
	}
}
