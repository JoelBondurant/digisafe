mod gui;
mod logger;
mod security;
mod storage;

pub fn main() -> gui::Result {
	logger::info("DigiSafe started.");
	security::preflight();
	gui::run()
}
