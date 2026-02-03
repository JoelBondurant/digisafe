pub mod colors;
pub mod components;
pub mod messages;

use crate::storage::{database::Database, entry::PasswordEntry, persistence};
use iced::{application, widget::text_editor, window, Element, Size, Task};
use messages::Message;

enum AppState {
	Locked {
		db_name: String,
		db_password: String,
		is_processing: bool,
	},
	Unlocked {
		query: String,
		password_entry: PasswordEntry,
		note: text_editor::Content,
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
	application(State::new, State::update, State::view)
		.theme(components::theme())
		.title(APP_NAME)
		.window(window::Settings {
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
				db_password: "".into(),
				is_processing: false,
			},
		}
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match &mut self.app_state {
			AppState::Locked {
				db_name,
				db_password,
				is_processing,
			} => match message {
				Message::AttemptUnlock => {
					*is_processing = true;
					let db_name_clone = db_name.clone();
					let db_password_clone = db_password.clone();
					return Task::perform(
						async move { persistence::load(db_name_clone, db_password_clone) },
						Message::UnlockResult,
					);
				}
				Message::UnlockResult(db) => {
					self.app_state = AppState::Unlocked {
						query: "".into(),
						password_entry: PasswordEntry::default(),
						note: text_editor::Content::new(),
						status: "Unlocked".into(),
						db,
					};
				}
				Message::DbNameChanged(new_db_name) => {
					*db_name = new_db_name;
				}
				Message::DbPasswordChanged(new_password) => {
					*db_password = new_password;
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
				password_entry,
				note,
				status,
				db,
			} => match message {
				Message::QueryInput(new_text) => {
					*query = new_text;
				}
				Message::QuerySubmit => {
					return Task::done(Message::PasswordEntryGet);
				}
				Message::PasswordEntryNameInput(new_text) => {
					password_entry.set_name(&new_text);
				}
				Message::PasswordEntryUsernameInput(new_text) => {
					password_entry.set_username(&new_text);
				}
				Message::PasswordEntryPasswordInput(new_text) => {
					password_entry.set_password(&new_text);
				}
				Message::PasswordEntryNoteAction(action) => {
					note.perform(action);
				}
				Message::PasswordEntryGet => {
					if let Some(found_password_entry) = db.get_password_entry(query) {
						*status = "Entry retrieved.".to_string();
						*password_entry = found_password_entry.clone();
						*note = text_editor::Content::with_text(found_password_entry.get_note());
					} else {
						*status = "Entry not retrieved.".to_string();
						*password_entry = PasswordEntry::default();
						*note = text_editor::Content::new();
					};
				}
				Message::PasswordEntrySet => {
					let name = password_entry.get_name();
					let note_string = note.text();
					if !name.is_empty() {
						if let Some(mut old_entry) = db.get_password_entry(name) {
							old_entry.set_username(password_entry.get_username());
							old_entry.set_password(password_entry.get_password());
							old_entry.set_note(&note_string);
							db.set_password_entry(old_entry);
							*status = "Entry set.".to_string();
						} else {
							let mut new_entry = PasswordEntry::default();
							new_entry.set_name(name);
							new_entry.set_username(password_entry.get_username());
							new_entry.set_password(password_entry.get_password());
							new_entry.set_note(&note_string);
							db.set_password_entry(new_entry.clone());
							*password_entry = new_entry;
							*status = "New entry set.".to_string();
						}
					} else {
						*status = "Name cannot be empty.".to_string();
					}
				}
				Message::Save => {
					let db_clone = db.clone();
					return Task::perform(
						async move { persistence::save(db_clone) },
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
				db_password,
				is_processing,
			} => components::unlock_screen(db_name, db_password, is_processing),
			AppState::Unlocked {
				query,
				password_entry,
				note,
				status,
				db: _,
			} => components::password_screen(query, password_entry, note, status),
		}
	}
}
