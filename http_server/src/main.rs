use ctrlc;
use http_server::Server;
use std::{
	io::Write,
};

use tokio::{runtime::Runtime,self, sync::broadcast};

const PORT: u16 = 7878;

fn main() {
	let rt = match Runtime::new() {
		Ok(rt) => rt,
		Err(err) => panic!("unable to start server's async runtime: {err}"),
	};

	rt.block_on(async {
		// wait for ctrl-c
		let (tx, sig) = broadcast::channel(1);
		let mut rx = tx.subscribe();

		let server = match Server::serve(PORT, sig) {
			Ok(s) => s,
			Err(e) => {
				eprintln!("Error starting server: {}", e);
				std::process::exit(1);
			}
		};

		println!("Server started on port {}", PORT);
		ctrlc::set_handler(move || {
			match tx.send(false) {
				Ok(_) => (),
				Err(err) => panic!("unable to send graceful shutdown signal, forcing shutdown: {err}"),
			};
		})
		.expect("Error setting Ctrl-C handler");

		rx.recv().await.expect("something went wrong while waiting to receive graceful shutdown signal");

		std::io::stdout().flush().unwrap();
		drop(server);
	});
	println!("Server gracefully stopped");
}
