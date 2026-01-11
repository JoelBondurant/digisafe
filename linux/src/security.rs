/*
* Todo:
* rustix::mm::memlockall
* check for secure boot parameters
* check total memory encryption
* check for full disk encryption
* apparmor or selinux setup
* additional os level application
*/

use rustix::{
	mm::{mlockall, MlockAllFlags},
	process::{getrlimit, set_dumpable_behavior, DumpableBehavior, Resource},
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
	let _ = set_dumpable_behavior(DumpableBehavior::NotDumpable);
}

fn verify_session_type() {
	let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
	if session_type != "wayland" {
		panic!("SECURITY VIOLATION: XDG_SESSION_TYPE must be set to wayland.");
	}
}

pub fn force_secure_backend() {
	if env::var("WINIT_UNIX_BACKEND").ok().as_deref() != Some("wayland") {
		unsafe {
			env::set_var("WINIT_UNIX_BACKEND", "wayland");
		}
	}
}

fn check_env(var: &str) {
	if std::env::var(var).map(|v| !v.is_empty()).unwrap_or(false) {
		panic!("SECURITY VIOLATION: {} is set.", var);
	}
}

fn enforce_no_preload() {
	check_env("LD_PRELOAD");
	check_env("LD_AUDIT");
}

pub fn preflight() {
	lock_memory_pages();
	set_not_dumpable();
	force_secure_backend();
	verify_session_type();
	enforce_no_preload();
}
