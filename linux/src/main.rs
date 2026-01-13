mod crypto;
mod gui;
mod security;
mod storage;

pub fn main() -> gui::Result {
	security::preflight();
	gui::run()
}
