use crate::gui::components;
use crate::gui::messages::Message;
use crate::storage::{database::Database, entry::PasswordEntry, persistence};
use iced::{
	application, clipboard, widget::text_editor, window, Element, Size, Subscription, Task,
};
use std::{
	thread,
	time::{Duration, Instant},
};

struct LockedState {
	db_name: String,
	db_password: String,
	is_processing: bool,
}

struct UnlockedState {
	query: String,
	password_entry: PasswordEntry,
	is_password_visible: bool,
	last_copy_time: Option<Instant>,
	last_interaction_time: Option<Instant>,
	note: text_editor::Content,
	status: String,
	db: Database,
}

enum AppState {
	Locked(LockedState),
	Unlocked(UnlockedState),
}

pub const APP_NAME: &str = "DigiSafe";

pub type Result = iced::Result;

pub fn run() -> Result {
	application(new, update, view)
		.subscription(subscription)
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

fn new() -> AppState {
	AppState::Locked(LockedState {
		db_name: "".into(),
		db_password: "".into(),
		is_processing: false,
	})
}

fn update(app_state: &mut AppState, message: Message) -> Task<Message> {
	match app_state {
		AppState::Locked(_) => update_locked(message, app_state),
		AppState::Unlocked(_) => update_unlocked(message, app_state),
	}
}

fn subscription(app_state: &AppState) -> Subscription<Message> {
	if let AppState::Unlocked(_) = &app_state {
		iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
	} else {
		Subscription::none()
	}
}

fn view(app_state: &AppState) -> Element<'_, Message> {
	match app_state {
		AppState::Locked(LockedState {
			db_name,
			db_password,
			is_processing,
		}) => components::unlock_screen(db_name, db_password, is_processing),
		AppState::Unlocked(UnlockedState {
			query,
			password_entry,
			is_password_visible,
			last_copy_time: _,
			last_interaction_time: _,
			note,
			status,
			db: _,
		}) => components::password_screen(query, password_entry, is_password_visible, note, status),
	}
}

fn update_locked(message: Message, app_state: &mut AppState) -> Task<Message> {
	let AppState::Locked(LockedState {
		db_name,
		db_password,
		is_processing,
	}) = app_state
	else {
		unreachable!("update_locked called with non-Locked state")
	};
	match message {
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
			*app_state = AppState::Unlocked(UnlockedState {
				query: "".into(),
				password_entry: PasswordEntry::default(),
				is_password_visible: false,
				last_copy_time: None,
				last_interaction_time: None,
				note: text_editor::Content::new(),
				status: "Unlocked".into(),
				db,
			});
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
	}
	Task::none()
}

fn update_unlocked(message: Message, app_state: &mut AppState) -> Task<Message> {
	let AppState::Unlocked(UnlockedState {
		query,
		password_entry,
		is_password_visible,
		last_copy_time,
		last_interaction_time,
		note,
		status,
		db,
	}) = app_state
	else {
		unreachable!("update_unlocked called with locked state")
	};
	if !matches!(message, Message::Tick) {
		*last_interaction_time = Some(Instant::now());
	}
	match message {
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
		Message::TogglePasswordVisibility => {
			*is_password_visible = !*is_password_visible;
		}
		Message::CopyPassword => {
			*last_copy_time = Some(Instant::now());
			*status = "Password copied.".to_string();
			return clipboard::write(password_entry.get_password().to_string());
		}
		Message::ClearClipboard => {
			*status = "Clearing clipboard...".to_string();
			thread::spawn(|| {
				use arboard::{Clipboard, LinuxClipboardKind as LCK, SetExtLinux};
				let mut cb = Clipboard::new().unwrap();
				for idx in 0..60 {
					let mut noise = [0u8; 16];
					getrandom::fill(&mut noise).unwrap();
					let munge = format!(
						"SCORTCHED_EARTH_{:03}_{:040}",
						idx,
						u128::from_le_bytes(noise)
					);
					let _ = cb.set().clipboard(LCK::Clipboard).text(&munge);
					let _ = cb.set().clipboard(LCK::Primary).text(munge);
					thread::sleep(Duration::from_millis(4));
				}
				let _ = cb.set().clipboard(LCK::Clipboard).text("");
				let _ = cb.set().clipboard(LCK::Primary).text("");
				thread::sleep(Duration::from_millis(20));
			});
			*last_copy_time = None;
			*status = "Clipboard cleared.".to_string();
		}
		Message::Tick => {
			if let Some(lit) = last_interaction_time {
				let idle_timeout =
					Duration::from_secs(if cfg!(debug_assertions) { 60 } else { 240 });
				let idle_elapsed = lit.elapsed();
				let idle_remaining = idle_timeout.saturating_sub(idle_elapsed).as_secs();
				if idle_remaining <= 10 {
					*status = format!("Idle time remaining: {idle_remaining}s");
				}
				if idle_elapsed >= idle_timeout {
					db.zeroize();
					return Task::done(Message::Lock);
				}
			}
			let copy_timeout = Duration::from_secs(if cfg!(debug_assertions) { 8 } else { 20 });
			if let Some(lct) = *last_copy_time {
				let copy_elapsed = lct.elapsed();
				if copy_elapsed >= Duration::from_secs(4) {
					let copy_remaining = copy_timeout.saturating_sub(copy_elapsed).as_secs();
					*status = format!("Password copy time remaining: {copy_remaining}s");
				}
				if copy_elapsed >= copy_timeout {
					return Task::done(Message::ClearClipboard);
				}
			}
		}
		Message::Lock => {
			*app_state = AppState::Locked(LockedState {
				db_name: "".into(),
				db_password: "".into(),
				is_processing: false,
			});
		}
		_ => {}
	}
	Task::none()
}
