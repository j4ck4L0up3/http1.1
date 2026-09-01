pub mod http_error;
pub mod method;

use http_error::HttpParseError;
use method::Method;
use std::io::{prelude::*, BufReader};

const HTTP_VERSION: &str = "HTTP/1.1";

struct Request {
	request_line: RequestLine,
}

impl Request {
	pub fn from_reader<T: Read>(mut reader: BufReader<T>) -> Result<Request, HttpParseError> {
		let mut curr_line = String::new();
		let mut part = String::new();

		loop {
			let _ = reader.read_line(&mut curr_line);

			if curr_line.is_empty() {
				break;
			}

			if curr_line.contains("Content-Length") {
				let header: Vec<&str> = curr_line.split(" ").collect();
				if let Some(length) = header[1].strip_suffix("\r\n") {
					let content_len: usize = match length.parse() {
						Ok(len) => len,
						Err(_) => return Err(HttpParseError::BadHeader(curr_line)),
					};

					let mut body = vec![0u8; content_len];
					let _ = reader.read_exact(&mut body);
					let _body_str = String::from_utf8(body);
				}
				break;
			}

			if curr_line.contains("\n") {
				let part_end = match curr_line.find("\n") {
					Some(idx) => idx,
					None => break,
				};

				let mut segment = curr_line;
				curr_line = segment.split_off(part_end + 1); // split_off is not inclusive
				part.push_str(segment.as_str());
			}
		}

		let lines: Vec<&str> = part.split("\r\n").collect();
		let raw_req_line: Vec<&str> = lines[0].split(" ").collect();

		let method = match Method::parse(raw_req_line[0]) {
			Ok(m) => m,
			Err(err) => return Err(err),
		};

		let mut request_target = String::new();
		if raw_req_line[1].contains("/") {
			request_target.push_str(raw_req_line[1]);
		} else {
			return Err(HttpParseError::MissingRequestTarget);
		}

		let mut http_version = String::new();
		if raw_req_line[2] == HTTP_VERSION {
			http_version.push_str(HTTP_VERSION);
		} else {
			return Err(HttpParseError::MissingHttpVersion);
		}

		let request_line = RequestLine {
			method,
			request_target,
			http_version,
		};

		Ok(Request { request_line })
	}
}

struct RequestLine {
	method: Method,
	request_target: String,
	http_version: String,
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use super::*;

	#[test]
	fn good_request_line() {
		let request_line =
			b"GET / HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let reader = BufReader::new(Cursor::new(request_line));

		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => panic!("expected request, got error: {err}"),
		};

		assert_eq!(Method::GET, request.request_line.method);
		assert_eq!("/", request.request_line.request_target);
		assert_eq!("HTTP/1.1", request.request_line.http_version);
	}

	#[test]
	fn good_request_line_with_path() {
		let request_line =
			b"GET /coffee HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let reader = BufReader::new(Cursor::new(request_line));

		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => panic!("expected request, got error: {err}"),
		};

		assert_eq!(Method::GET, request.request_line.method);
		assert_eq!("/coffee", request.request_line.request_target);
		assert_eq!("HTTP/1.1", request.request_line.http_version);
	}

	#[test]
	fn missing_method_in_request() {
		let request_line =
			b"/coffee HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let reader = BufReader::new(Cursor::new(request_line));

		match Request::from_reader(reader) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingMethod, err)
			}
		}
	}

	#[test]
	fn good_post_request() {
		let request_line = b"POST /coffee HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: 22\r\n{\"flavor\":\"dark mode\"}";
		let reader = BufReader::new(Cursor::new(request_line));

		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => panic!("expected request, got error: {err}"),
		};

		assert_eq!(Method::POST, request.request_line.method);
		assert_eq!("/coffee", request.request_line.request_target);
		assert_eq!(
			String::from(HTTP_VERSION),
			request.request_line.http_version
		);
	}
}
