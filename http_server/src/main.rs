use ctrlc;
use http_server::Server;
use std::{
	io::Write,
	sync::{atomic::AtomicBool},
};

use tokio::{runtime::Runtime, self};

const PORT: u16 = 7878;

fn main() {
	let rt = match Runtime::new() {
		Ok(rt) => rt,
		Err(err) => panic!("unable to start server's async runtime: {err}"),
	};

	rt.block_on(async {
		// wait for ctrl-c
		let (tx, rx) = tokio::sync::mpsc::channel(1);

		let server = match Server::serve(PORT, rx) {
			Ok(s) => s,
			Err(e) => {
				eprintln!("Error starting server: {}", e);
				std::process::exit(1);
			}
		};

		println!("Server started on port {}", PORT);
		ctrlc::set_handler(move || {
			match tx.blocking_send(AtomicBool::new(false)) {
				Ok(_) => (),
				Err(err) => panic!("unable to send graceful shutdown signal, forcing shutdown: {err}"),
			};
		})
		.expect("Error setting Ctrl-C handler");

		std::io::stdout().flush().unwrap();
		drop(server);
	});
	println!("Server gracefully stopped");
}
