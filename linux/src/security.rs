/*
* Todo:
* check for tpm boot parameters
* check total memory encryption
* check for full disk encryption
* apparmor or selinux setup
*/

use rustix::{
	mm::{mlockall, MlockAllFlags},
	process::{getrlimit, set_dumpable_behavior, setrlimit, DumpableBehavior, Resource, Rlimit},
};
use std::env;

pub fn get_memory_lock_limits() -> String {
	let limits = getrlimit(Resource::Memlock);
	match (limits.current, limits.maximum) {
		(Some(limit_current), Some(limit_max)) => format!(
			"{} bytes current, {} bytes maximum",
			limit_current, limit_max
		),
		_ => "Memlock limit error.".to_string(),
	}
}

fn lock_memory_pages() {
	if mlockall(MlockAllFlags::CURRENT | MlockAllFlags::FUTURE).is_err() {
		eprintln!("Memory lock failure.");
		eprintln!("Memory lock limits: {}.", get_memory_lock_limits());
	}
}

fn set_not_dumpable() {
	let _ = setrlimit(
		Resource::Core,
		Rlimit {
			current: Some(0),
			maximum: Some(0),
		},
	);
	let _ = set_dumpable_behavior(DumpableBehavior::NotDumpable);
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
