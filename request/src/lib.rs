pub mod http_error;
pub mod method;

use http_error::HttpParseError;
use method::Method;
use std::io::{self, BufReader, Cursor, prelude::*};

const HTTP_VERSION: &str = "1.1";
const BUFFER_SIZE: usize = 8;

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
		let mut request = Request::new(); 

		let mut buffer: Vec<u8> = vec![0u8; BUFFER_SIZE];
		let mut read_idx: usize = 0;
		let mut shift: usize = 1;

		while request.state != ParseState::Done {

			let read = match reader.read(&mut buffer[read_idx..]) {
				Ok(n) => n,
				Err(_) => return Err(HttpParseError::RequestLineParseError),
			};

			if read == 0 {
				break;
			}
			
			read_idx += read;

			let parsed = match &mut request.parse(&mut buffer) {
				Ok(p) => *p,
				Err(err) => return Err(*err),
			};

			if parsed == 0 && read_idx == buffer.len() {
				let buffer_size = BUFFER_SIZE << shift;
				buffer.resize(buffer_size, 0);
				shift += 1;
			}

			if parsed > 0 {
				read_idx -= parsed;

			}
			
		}

		Ok(request)
	}

	fn new() -> Request {
		Request { request_line: None, state: ParseState::Initialized }
	}

	fn parse(&mut self, data: &mut Vec<u8>) -> Result<usize, HttpParseError> {
		let mut bytes_read: usize = 0;

		if self.state == ParseState::Initialized {
			let parsed_req_line: (Option<RequestLine>, usize) = match RequestLine::parse(data) {
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
	fn parse(data: &mut Vec<u8>) -> Result<(Option<RequestLine>, usize), HttpParseError> {
		let mut parsed: usize = 0;
		let buf = match String::from_utf8(data.to_vec()) {
			Ok(b) => b,
			Err(_) => return Err(HttpParseError::RequestLineParseError),
		};

		if !buf.contains("\r\n") {
			return Ok((None, 0));
		}

		let parts: Vec<&str> = buf.split("\r\n").collect();
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

		parsed += parts[0].len();

		Ok((Some(request_line), parsed))
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
