use request::{http_error::HttpParseError, method::Method, Request, HTTP_VERSION};
use std::io::{BufReader, Cursor};

fn main() {
	let request_line =
		b"POST /submit HTTP/1.1\r\nHost: localhost:42069\r\nContent-Length: 20\r\n\r\npartial content";

	let reader = BufReader::new(Cursor::new(request_line));

	let _ = match Request::from_reader(reader) {
		Ok(req) => panic!("expected error, received: {req:?}"),
		Err(err) => assert_eq!(HttpParseError::InvalidPartialContent, err),
	};
}
