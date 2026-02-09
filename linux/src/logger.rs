use std::io::{self, IsTerminal, Write};

fn log(level_prefix: &str, level_label: &str, msg: &str) {
	let level_prefix = if io::stderr().is_terminal() {
		""
	} else {
		level_prefix
	};
	let mut stderr = io::stderr();
	let _ = writeln!(stderr, "{level_prefix}{level_label}: {msg}");
	let _ = stderr.flush();
}

pub fn info(msg: &str) {
	log("<6>", "Info", msg);
}

pub fn warn(msg: &str) {
	log("<4>", "Warn", msg);
}

pub fn error(msg: &str) {
	log("<3>", "Error", msg);
}

pub fn critical(msg: &str) {
	log("<2>", "Critical", msg);
}
