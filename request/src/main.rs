use request::{http_error::HttpParseError, method::Method, Request, HTTP_VERSION};
use std::io::{BufReader, Cursor};

fn main() {
	let request_line = b"GET / HTTP/1.1\r\nHost: localhost:8080\r\n\r\n";

	let reader = BufReader::new(Cursor::new(request_line));

	let _request = match Request::from_reader(reader) {
		Ok(req) => req,
		Err(err) => panic!("expected parsed request, received error: {err:?}"),
	};

	// let req_line = request.request_line.unwrap();
	// let headers = request.headers;

	// assert_eq!(Method::GET, req_line.method);
	// assert_eq!("/", req_line.request_target);
	// assert_eq!(HTTP_VERSION, req_line.http_version);
	// assert_eq!(headers.get("host"), "localhost:7878");
	// assert_eq!(headers.get("user-agent"), "curl/7.81.0");
	// assert_eq!(headers.get("accept"), "*/*");
}
