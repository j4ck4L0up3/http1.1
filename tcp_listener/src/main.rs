use tokio::runtime::Runtime;

fn main() {
	let rt = match Runtime::new() {
		Ok(run) => run,
		Err(err) => panic!("Could not start async runtime: {err}"),
	};
	tcp_listener::serve(rt);
}
