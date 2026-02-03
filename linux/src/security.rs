/*
* Todo:
* check for tpm boot parameters
* check total memory encryption
* check for full disk encryption
* apparmor or selinux setup
*/

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
			"Memlock limit error.".to_string()
		}
	}
}

fn lock_memory_pages() {
	unsafe {
		let flags = MCL_CURRENT | MCL_FUTURE;
		if mlockall(flags) != 0 {
			eprintln!("Memory lock failure.");
			eprintln!("Memory lock limits: {}.", get_memory_lock_limits());
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
		panic!("SECURITY VIOLATION: XDG_SESSION_TYPE must be set to wayland.");
	}
}

fn check_env(var: &str) {
	if env::var(var).map(|v| !v.is_empty()).unwrap_or(false) {
		panic!("SECURITY VIOLATION: {} is set.", var);
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
}
