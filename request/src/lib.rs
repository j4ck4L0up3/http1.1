pub mod http_error;
pub mod method;

use http_error::HttpParseError;
use method::Method;
use std::io::{BufReader, Cursor, prelude::*};

const HTTP_VERSION: &str = "1.1";

#[derive(PartialEq, Clone, Debug)]
enum ParseState {
	Initialized,
	Done,
}

struct Request {
	request_line: Option<RequestLine>,
	state: ParseState,
}

impl Request {
	pub fn from_reader<T: Read>(mut reader: BufReader<T>) -> Result<Request, HttpParseError> {

		Ok(Request { request_line })
		/* let mut curr_line = String::new();
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
		let raw_req_line: Vec<&str> = lines[0].split(" ").collect(); */

	}

	fn new() -> Request {
		Request { request_line: None, state: ParseState::Initialized }
	}

	fn parse(mut self, data: &[u8]) -> Result<usize, HttpParseError> {
		let cursor = Cursor::new(data);
		let reader = BufReader::new(cursor);

		let mut bytes_read: usize = 0;

		if self.state == ParseState::Initialized {
			let parsed_req_line: (Option<RequestLine>, usize) = match RequestLine::parse(reader) {
				Ok(p) => p,
				Err(err) => return Err(err),
			};
			
			if parsed_req_line.1 == 0 {
				return Ok(0);
			}

			assert!(parsed_req_line.0 != None);

			bytes_read += parsed_req_line.1;
			self.request_line = parsed_req_line.0;
			self.state = ParseState::Done;
		} else if self.state == ParseState::Done {
			return Err(HttpParseError::ReadingDoneParser);
		} else {
			return Err(HttpParseError::UnknownParserState);
		}

		Ok(bytes_read)
	}
}

#[derive(PartialEq, Debug)]
struct RequestLine {
	method: Method,
	request_target: String,
	http_version: String,
}

impl RequestLine {
	fn parse<T: Read>(mut reader: BufReader<T>) -> Result<(Option<RequestLine>, usize), HttpParseError> {
		let mut buf = String::new();
		let bytes_read = reader.read_line(&mut buf).unwrap_or_default();

		if bytes_read == 0 {
			return Ok((None, 0));
		}

		if !buf.contains("\r\n") {
			return Ok((None, 0));
		}

		let raw_req_line: Vec<&str> = buf.split(" ").collect();

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

		let request_line = RequestLine {
			method,
			request_target,
			http_version,
		};

		Ok((Some(request_line), bytes_read))
	}
}

#[cfg(test)]
mod tests {
	use std::io::{self, Cursor, Read };

	use super::*;

	struct ChunkReader<'a> {
		data: &'a[u8],
		bytes_per_read: usize,
		pos: usize,
	}

	impl<'a> Read for ChunkReader<'a> {
		fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
			if self.pos >= self.data.len() {
				return Ok(0);
			}
			
			let mut end_index = self.pos + self.bytes_per_read;
			if end_index > self.data.len() {
				end_index = self.data.len();
			}
		
			let chunk = &self.data[self.pos..end_index];
			let n = chunk.len();
			self.pos += n;
			buf[..n].copy_from_slice(&chunk);

			Ok(n)
		}
	}

	#[test]
	fn good_request_line() {
		let request_line =
			b"GET / HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";

		for i in 1..request_line.len() {
			let reader = ChunkReader {
				data: request_line,
				bytes_per_read: i,
				pos: 0,
			};
			let reader = BufReader::new(reader);

			let request = match Request::from_reader(reader) {
				Ok(req) => req,
				Err(err) => panic!("expected request, got error: {err}"),
			};

			let req_line = request.request_line.unwrap();

			assert_eq!(Method::GET, req_line.method);
			assert_eq!("/", req_line.request_target);
			assert_eq!(HTTP_VERSION, req_line.http_version);
		}
	}

	#[test]
	fn good_request_line_with_path() {
		let request_line =
			b"GET /coffee HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";

		for i in 1..request_line.len() {
			let reader = ChunkReader {
				data: request_line,
				bytes_per_read: i,
				pos: 0,
			};
			let reader = BufReader::new(reader);

			let request = match Request::from_reader(reader) {
				Ok(req) => req,
				Err(err) => panic!("expected request, got error: {err}"),
			};

			let req_line = request.request_line.unwrap();

			assert_eq!(Method::GET, req_line.method);
			assert_eq!("/coffee", req_line.request_target);
			assert_eq!(HTTP_VERSION, req_line.http_version);
		}
	}

	#[test]
	fn good_post_request() {
		let request_line = 
			b"POST /coffee HTTP/1.1\r\nhost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: 22\r\n{\"flavor\":\"dark mode\"}";

		for i in 1..request_line.len() {
			let reader = ChunkReader {
				data: request_line,
				bytes_per_read: i,
				pos: 0,
			};
			let reader = BufReader::new(reader);

			let request = match Request::from_reader(reader) {
				Ok(req) => req,
				Err(err) => panic!("expected request, got error: {err}"),
			};

			let req_line = request.request_line.unwrap();

			assert_eq!(Method::POST, req_line.method);
			assert_eq!("/coffee", req_line.request_target);
			assert_eq!(
				String::from(HTTP_VERSION),
				req_line.http_version
			);
		}
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
	fn missing_target_in_request() {
		let request_line =
			b"GET HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let reader = BufReader::new(Cursor::new(request_line));

		match Request::from_reader(reader) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingRequestTarget, err)
			}
		}
	}

	#[test]
	fn missing_version_in_request() {
		let request_line =
			b"GET /coffee \r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let reader = BufReader::new(Cursor::new(request_line));

		match Request::from_reader(reader) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingHttpVersion, err)
			}
		}
	}

	#[test]
	fn request_line_out_of_order() {
		let request_line1 =
			b"/coffee GET HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let request_line2 =
			b"GET HTTP/1.1 /coffee\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let request_line3 =
			b"HTTP/1.1 GET /coffee\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let request_line4 =
			b"HTTP/1.1 /coffee GET\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let request_line5 =
			b"/coffee HTTP/1.1 GET\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";

		let reader1 = BufReader::new(Cursor::new(request_line1));
		let reader2 = BufReader::new(Cursor::new(request_line2));
		let reader3 = BufReader::new(Cursor::new(request_line3));
		let reader4 = BufReader::new(Cursor::new(request_line4));
		let reader5 = BufReader::new(Cursor::new(request_line5));

		match Request::from_reader(reader1) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingMethod, err)
			}
		}

		match Request::from_reader(reader2) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingRequestTarget, err)
			}
		}

		match Request::from_reader(reader3) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingMethod, err)
			}
		}

		match Request::from_reader(reader4) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingMethod, err)
			}
		}

		match Request::from_reader(reader5) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::MissingMethod, err)
			}
		}
	}

	#[test]
	fn invalid_http_version() {
		let request_line =
			b"GET /coffee HTTP/2\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n";
		let reader = BufReader::new(Cursor::new(request_line));

		match Request::from_reader(reader) {
			Ok(_) => panic!("expected error, got ok"),
			Err(err) => {
				assert_eq!(HttpParseError::WrongHttpVersion, err)
			}
		}
	}
}
