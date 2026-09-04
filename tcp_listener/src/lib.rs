use request::{ParseState, Request};
use std::{io::BufReader, net::TcpListener};

pub fn serve() {
	let listener = match TcpListener::bind("127.0.0.1:7878") {
		Ok(l) => l,
		Err(err) => panic!("Could not start TCP server {err:?}"),
	};

	for stream in listener.incoming() {
		let stream = match stream {
			Ok(s) => s,
			Err(err) => panic!("Could not connect to TCP stream {err:?}"),
		};

		println!("Connection received!");

		let reader = BufReader::new(stream);
		let request = match Request::from_reader(reader) {
			Ok(req) => req,
			Err(err) => panic!("Error with parsed request {err}"),
		};

		let request_line = match request.request_line {
			Some(line) => line,
			None => panic!("No request line found"),
		};

		println!("{request_line}");

		if request.state == ParseState::Done {
			break;
		}
	}

	println!("Connection closed!");
}

#[cfg(test)]
mod tests {
	use super::*;
}
