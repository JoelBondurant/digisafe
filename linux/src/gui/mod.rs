pub mod colors;
pub mod components;
pub mod messages;

use crate::storage::persistent;
use crate::storage::volatile::Database;
use iced::{widget::text_editor, window, Element, Size, Task};
use messages::Message;

enum AppState {
	Locked {
		db_name: String,
		password: String,
		is_processing: bool,
	},
	Unlocked {
		query: String,
		value: text_editor::Content,
		status: String,
		db: Database,
	},
}

pub const APP_NAME: &str = "DigiSafe";

struct State {
	app_state: AppState,
}

pub type Result = iced::Result;

pub fn run() -> Result {
	iced::application(State::new, State::update, State::view)
		.theme(components::theme())
		.title(APP_NAME)
		.window(iced::window::Settings {
			size: Size::new(1000.0, 800.0),
			position: window::Position::Centered,
			decorations: false,
			transparent: false,
			..Default::default()
		})
		.run()
}

impl State {
	pub fn new() -> Self {
		Self {
			app_state: AppState::Locked {
				db_name: "".into(),
				password: "".into(),
				is_processing: false,
			},
		}
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match &mut self.app_state {
			AppState::Locked {
				db_name,
				password,
				is_processing,
			} => match message {
				Message::AttemptUnlock => {
					*is_processing = true;
					let db_name_clone = db_name.clone();
					let master_password = password.clone();
					return Task::perform(
						async move { persistent::load(db_name_clone, master_password) },
						Message::UnlockResult,
					);
				}
				Message::UnlockResult(db) => {
					self.app_state = AppState::Unlocked {
						query: "".into(),
						value: text_editor::Content::new(),
						status: "Unlocked".into(),
						db,
					};
				}
				Message::DbNameChanged(new_db_name) => {
					*db_name = new_db_name;
				}
				Message::PasswordChanged(new_password) => {
					*password = new_password;
				}
				Message::CloseWindow => {
					return window::latest().and_then(window::close);
				}
				Message::DragWindow => {
					return window::latest().and_then(window::drag);
				}
				_ => {}
			},
			AppState::Unlocked {
				query,
				value,
				status,
				db,
			} => match message {
				Message::QueryInput(new_text) => {
					*query = new_text;
				}
				Message::QuerySubmit => {
					return Task::done(Message::Get);
				}
				Message::ValueAction(action) => {
					value.perform(action);
				}
				Message::Get => {
					if let Some(found_value) = db.get_private(query) {
						*status = "Entry retrieved.".to_string();
						*value = text_editor::Content::with_text(&found_value);
					} else {
						*status = "Entry not retrieved.".to_string();
						*value = text_editor::Content::new();
					};
				}
				Message::Set => {
					let content_string = value.text();
					if !query.is_empty() {
						db.set_private(query.clone(), content_string);
						*status = "Entry set.".to_string();
					} else {
						*status = "Query was empty.".to_string();
					}
				}
				Message::Save => {
					let db_clone = db.clone();
					return Task::perform(
						async move { persistent::save(db_clone) },
						Message::SaveResult,
					);
				}
				Message::SaveResult(msg) => {
					*status = msg;
				}
				Message::CloseWindow => {
					return window::latest().and_then(window::close);
				}
				Message::DragWindow => {
					return window::latest().and_then(window::drag);
				}
				_ => {}
			},
		}
		Task::none()
	}

	fn view(&self) -> Element<'_, Message> {
		match &self.app_state {
			AppState::Locked {
				db_name,
				password,
				is_processing,
			} => components::unlock_screen(db_name, password, is_processing),
			AppState::Unlocked {
				query,
				value,
				status,
				db: _,
			} => components::main_screen(query, value, status),
		}
	}
}
