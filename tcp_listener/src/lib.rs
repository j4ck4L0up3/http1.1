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

		println!(
			"Request line:\n- Method: {}\n- Target: {}\n- Version: {}\nHeaders:",
			request.method.unwrap(),
			request.request_target,
			request.http_version,
		);

		for key in request.headers.field_lines.keys() {
			println!("- {}: {}", &*key, request.headers.get(key));
		}

		if !request.body.is_empty() {
			println!(
				"Body:\n{}",
				std::str::from_utf8(&*request.body.to_vec()).unwrap()
			);
		}

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
