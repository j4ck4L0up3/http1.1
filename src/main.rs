use http1_1::get_lines_channel;
use std::net::TcpListener;

fn main() {
	let listener = match TcpListener::bind("127.0.0.1:7878") {
		Ok(l) => l,
		Err(err) => panic!("Could not start TCP server {err:?}"),
	};

	for stream in listener.incoming().take(1) {
		let stream = match stream {
			Ok(s) => s,
			Err(err) => panic!("Could not connect to TCP stream {err:?}"),
		};

		println!("Connection received!");
		let rx = get_lines_channel(stream);
		let msg = rx.recv();

		match msg {
			Ok(msg) => println!("{msg}"),
			Err(err) => panic!("Could not receive msg: {err}"),
		};
	}

	println!("Connection closed!");
}
