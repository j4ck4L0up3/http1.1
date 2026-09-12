pub mod http_error;
pub mod method;
pub mod headers;
mod request_line;

use method::Method;
use headers::Headers;
use http_error::HttpParseError;
use request_line::RequestLine;
use std::{
	io::{BufReader, prelude::*},
};

pub const HTTP_VERSION: &str = "1.1";
const BUFFER_SIZE: usize = 8;

#[derive(PartialEq, Clone, Debug)]
pub enum ParseState {
	Initialized,
	ParsingHeaders,
	Done,
}

#[derive(Debug)]
pub struct Request {
	pub method: Option<Method>,
	pub request_target: Box<str>,
	pub http_version: Box<str>,
	pub headers: Headers,
	pub state: ParseState,
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
				if request.state == ParseState::ParsingHeaders {
					return Err(HttpParseError::MissingEndOfHeaders);
				}
				break;
			}
			
			read_idx += read;

			let parsed = match &mut request.parse(&buffer[..read_idx]) {
				Ok(p) => *p,
				Err(err) => return Err(*err),
			};

			if parsed == 0 && read_idx == buffer.len() {
				let buffer_size = BUFFER_SIZE << shift;
				buffer.resize(buffer_size, 0);
				shift += 1;
			}

			if parsed > 0 {
				buffer.copy_within(parsed..read_idx, 0);
				read_idx -= parsed;
			}
			
		}

		Ok(request)
	}

	fn new() -> Request {
		Request { 
			method: None,
			request_target: String::new().into_boxed_str(),
			http_version: String::new().into_boxed_str(),
			headers: Headers::new(),
			state: ParseState::Initialized
		}
	}

	fn parse(&mut self, data: &[u8]) -> Result<usize, HttpParseError> {
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
			
			if let Some(request_line) = parsed_req_line.0 {
				self.method = request_line.method;
				self.request_target = request_line.request_target;
				self.http_version = request_line.http_version;
			}

			self.state = ParseState::ParsingHeaders;
		} else if self.state == ParseState::ParsingHeaders {
			let results: (usize, bool) = match self.headers.parse(data) {
				Ok(tup) => tup,
				Err(err) => return Err(err),
			};
			
			bytes_read += results.0;
			let done = results.1;

			if done && bytes_read > 0 {
				self.state = ParseState::Done;
			}
		} else if self.state == ParseState::Done {
			return Err(HttpParseError::ReadingDoneParser);
		} else {
			return Err(HttpParseError::UnknownParserState);
		}

		Ok(bytes_read)
	}
}

#[cfg(test)]
mod tests {
	use std::io::{self, Cursor, Read };
	use method::Method;

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

			let headers = request.headers;

			assert_eq!(Method::GET, request.method.unwrap());
			assert_eq!("/", &*request.request_target);
			assert_eq!(HTTP_VERSION, &*request.http_version);
			assert_eq!(headers.get("host"), "localhost:7878");
			assert_eq!(headers.get("user-agent"), "curl/7.81.0");
			assert_eq!(headers.get("accept"), "*/*");
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

			let headers = request.headers;

			assert_eq!(Method::GET, request.method.unwrap());
			assert_eq!("/coffee", &*request.request_target);
			assert_eq!(HTTP_VERSION, &*request.http_version);
			assert_eq!(headers.get("host"), "localhost:7878");
			assert_eq!(headers.get("user-agent"), "curl/7.81.0");
			assert_eq!(headers.get("accept"), "*/*");
		}
	}

	#[test]
	fn good_post_request_line() {
		let request_line = 
			b"POST /coffee HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: 22\r\n\r\n{\"flavor\":\"dark mode\"}";

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

			let headers = request.headers;

			assert_eq!(Method::POST, request.method.unwrap());
			assert_eq!("/coffee", &*request.request_target);
			assert_eq!(HTTP_VERSION, &*request.http_version);
			assert_eq!(headers.get("host"), "localhost:7878");
			assert_eq!(headers.get("user-agent"), "curl/7.81.0");
			assert_eq!(headers.get("accept"), "*/*");
			assert_eq!(headers.get("content-type"), "application/json");
			assert_eq!(headers.get("content-length"), "22");
		}
	}

	#[test]
	fn valid_duplicate_headers() {
		let request_line = 
			b"GET /coffee HTTP/1.1\r\nHost: localhost:7878\r\nSet-Cookie: very=cool\r\nSet-Cookie: nice=guy\r\n\r\n";

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

			let headers = request.headers;

			assert_eq!(Method::GET, request.method.unwrap());
			assert_eq!("/coffee", &*request.request_target);
			assert_eq!(HTTP_VERSION, &*request.http_version);
			assert_eq!(headers.get("host"), "localhost:7878");
			assert_eq!(headers.get("set-cookie"), "very=cool, nice=guy");
		}
	}

	#[test]
	fn valid_case_insensitive_headers() {
		let request_line = 
			b"GET /coffee HTTP/1.1\r\nHOST: localhost:7878\r\nset-CooKie: very=cool\r\nSET-cookie: nice=guy\r\n\r\n";

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

			let headers = request.headers;

			assert_eq!(Method::GET, request.method.unwrap());
			assert_eq!("/coffee", &*request.request_target);
			assert_eq!(HTTP_VERSION, &*request.http_version);

			// also testing get here for case insensitivity
			assert_eq!(headers.get("Host"), "localhost:7878"); 
			assert_eq!(headers.get("Set-Cookie"), "very=cool, nice=guy");
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

	#[test]
	fn malformed_header() {
		let request_line =
			b"GET / HTTP/1.1\r\nHost localhost:7878\r\n\r\n";

			let reader = ChunkReader {
				data: request_line,
				bytes_per_read: 3,
				pos: 0,
			};
			let reader = BufReader::new(reader);

			let _request = match Request::from_reader(reader) {
				Ok(req) => panic!("expected error, received: {req:?}"),
				Err(err) => assert_eq!(HttpParseError::InvalidHeaderWhitespace, err),
			};
	}

	#[test]
	fn empty_header() {
		let request_line =
			b"GET / HTTP/1.1\r\nHost: \r\n\r\n";

		let reader = BufReader::new(Cursor::new(request_line));

		let _request = match Request::from_reader(reader) {
			Ok(req) => panic!("expected error, received: {req:?}"),
			Err(err) => assert_eq!(HttpParseError::EmptyFieldValue, err),
		};
	}

	#[test]
	fn invalid_duplicate_headers() {
		let request_line1 =
			b"GET / HTTP/1.1\r\nHost: example.com\r\nHost: localhost:7878\r\n\r\n";
		let request_line2 =
			b"GET / HTTP/1.1\r\nContent-Length: 10\r\nContent-Length: 78\r\n\r\n";

		let reader1 = BufReader::new(Cursor::new(request_line1));
		let reader2 = BufReader::new(Cursor::new(request_line2));

		let _request1 = match Request::from_reader(reader1) {
			Ok(req) => panic!("expected error, received: {req:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
		
		let _request2 = match Request::from_reader(reader2) {
			Ok(req) => panic!("expected error, received: {req:?}"),
			Err(err) => assert_eq!(HttpParseError::InvalidDuplicateHeader, err),
		};
	}

	#[test]
	fn missing_end_of_headers() {
		let request_line =
			b"GET / HTTP/1.1\r\nHost: localhost:7878\r\n";

			let reader = ChunkReader {
				data: request_line,
				bytes_per_read: 3,
				pos: 0,
			};
			let reader = BufReader::new(reader);

			let _request = match Request::from_reader(reader) {
				Ok(req) => panic!("expected error, received: {req:?}"),
				Err(err) => assert_eq!(HttpParseError::MissingEndOfHeaders, err),
			};
	}
}
