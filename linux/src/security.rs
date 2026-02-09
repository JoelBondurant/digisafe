use crate::logger::{critical, info, warn};
use libc::{
	getrlimit, mlockall, prctl, rlimit, setrlimit, MCL_CURRENT, MCL_FUTURE, PR_SET_DUMPABLE,
	RLIMIT_CORE, RLIMIT_MEMLOCK,
};
use std::env;

pub fn get_memory_lock_limits() -> String {
	unsafe {
		let mut rlim = rlimit {
			rlim_cur: 0,
			rlim_max: 0,
		};
		if getrlimit(RLIMIT_MEMLOCK, &mut rlim) == 0 {
			format!(
				"{} bytes current, {} bytes maximum",
				rlim.rlim_cur, rlim.rlim_max
			)
		} else {
			let msg = "Memlock limit error.".to_string();
			warn(&msg);
			msg
		}
	}
}

fn lock_memory_pages() {
	unsafe {
		let flags = MCL_CURRENT | MCL_FUTURE;
		if mlockall(flags) != 0 {
			critical("Memory lock failure.");
			warn(&format!(
				"Memory lock limits: {}.",
				get_memory_lock_limits()
			));
			std::process::exit(1);
		}
	}
}

fn set_not_dumpable() {
	unsafe {
		let rlim = rlimit {
			rlim_cur: 0,
			rlim_max: 0,
		};
		setrlimit(RLIMIT_CORE, &rlim);
		prctl(PR_SET_DUMPABLE, 0, 0, 0, 0);
	}
}

pub fn force_secure_display() {
	if env::var("WAYLAND_DISPLAY").ok().as_deref() != Some("wayland-0") {
		unsafe {
			env::set_var("WAYLAND_DISPLAY", "wayland-0");
		}
	}
}

fn verify_secure_display() {
	let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
	if session_type != "wayland" {
		critical("SECURITY VIOLATION: XDG_SESSION_TYPE must be set to wayland.");
		std::process::exit(1);
	}
}

fn check_env(var: &str) {
	if env::var(var).map(|v| !v.is_empty()).unwrap_or(false) {
		critical(&format!("SECURITY VIOLATION: {} is set.", var));
		std::process::exit(1);
	}
}

fn enforce_no_preload() {
	check_env("LD_PRELOAD");
	check_env("LD_AUDIT");
}

pub fn preflight() {
	force_secure_display();
	lock_memory_pages();
	set_not_dumpable();
	enforce_no_preload();
	verify_secure_display();
	info("Preflight security checks passed.");
}
