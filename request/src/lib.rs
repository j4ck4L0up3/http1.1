use std::io::{prelude::*, BufReader};

struct Request {
	request_line: RequestLine,
}

impl Request {
	pub fn from_reader<T: Read>(_reader: BufReader<T>) -> Result<Request, HttpError> {
		todo!()
	}
}

struct RequestLine {
	method: Method,
	request_target: String,
	http_version: String,
}

#[derive(PartialEq, Clone, Debug)]
enum HttpError {
	MissingMethod,
	MissingRequestTarget,
	MissingHttpVersion,
}

#[derive(PartialEq, Debug)]
enum Method {
	GET,
	HEAD,
	POST,
	PUT,
	DELETE,
	CONNECT,
	OPTIONS,
	TRACE,
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

		// TODO: add proper error handling
		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => match err {
				HttpError::MissingMethod => panic!("Method not passed in request"),
				HttpError::MissingRequestTarget => panic!("URI request target not passed in request"),
				HttpError::MissingHttpVersion => panic!("HTTP version not passed in request"),
			},
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

		// TODO: add proper error handling
		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => match err {
				HttpError::MissingMethod => panic!("Method not passed in request"),
				HttpError::MissingRequestTarget => panic!("URI request target not passed in request"),
				HttpError::MissingHttpVersion => panic!("HTTP version not passed in request"),
			},
		};

		assert_eq!(Method::GET, request.request_line.method);
		assert_eq!("/coffee", request.request_line.request_target);
		assert_eq!("HTTP/1.1", request.request_line.http_version);
	}
}
