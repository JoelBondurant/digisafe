pub mod components;
pub mod messages;

use crate::storage::persistent;
use crate::storage::volatile::Database;
use iced::{
	border,
	theme::Theme,
	widget::{center, column, container, row, space, text, text_editor},
	window, Background, Center, Color, Element, Fill, Size, Task,
};
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
		.theme(components::black_hole_theme())
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
			} => {
				let unlock_panel = container(center(
					column![
						text("Unlock Database").size(16),
						components::styled_text_input("name...", db_name)
							.on_input(Message::DbNameChanged),
						components::styled_text_input("password...", password)
							.secure(true)
							.on_input(Message::PasswordChanged)
							.on_submit(Message::AttemptUnlock),
						components::styled_button(
							if *is_processing {
								"Unlocking"
							} else {
								"Unlock"
							},
							Message::AttemptUnlock
						)
					]
					.spacing(20)
					.width(600),
				))
				.width(Fill)
				.height(Fill);

				column![components::title_bar(), unlock_panel].into()
			}
			AppState::Unlocked {
				query,
				value,
				status,
				db: _,
			} => {
				let query_bar = components::styled_text_input("search...", query)
					.on_input(Message::QueryInput)
					.on_submit(Message::QuerySubmit);

				let header = container(query_bar).padding(4).width(Fill);

				let value_editor = text_editor(value)
					.id("value")
					.size(18)
					.height(Fill)
					.on_action(Message::ValueAction)
					.wrapping(text::Wrapping::Word)
					.style(|_theme: &Theme, status: text_editor::Status| match status {
						text_editor::Status::Focused { .. } => text_editor::Style {
							background: Background::Color(Color::from_rgb8(8, 4, 10)),
							border: border::Border {
								color: Color::from_rgb8(110, 10, 240),
								width: 2.0,
								radius: 5.0.into(),
							},
							placeholder: Color::from_rgb8(100, 90, 100),
							value: Color::from_rgb8(20, 250, 20),
							selection: Color::from_rgb8(110, 10, 240),
						},
						text_editor::Status::Hovered => text_editor::Style {
							background: Background::Color(Color::from_rgb8(10, 8, 16)),
							border: border::Border {
								color: Color::from_rgb8(80, 8, 140),
								width: 1.5,
								radius: 5.0.into(),
							},
							placeholder: Color::from_rgb8(80, 70, 80),
							value: Color::from_rgb8(200, 180, 200),
							selection: Color::from_rgb8(110, 10, 240),
						},
						_ => text_editor::Style {
							background: Background::Color(Color::from_rgb8(4, 4, 8)),
							border: border::Border {
								color: Color::from_rgb8(60, 8, 100),
								width: 1.0,
								radius: 5.0.into(),
							},
							placeholder: Color::from_rgb8(80, 70, 80),
							value: Color::from_rgb8(200, 180, 200),
							selection: Color::from_rgb8(110, 10, 240),
						},
					});

				let main_content = container(center(column![value_editor].spacing(20)))
					.padding(4)
					.width(Fill);

				let button_bar = row![
					space::horizontal(),
					components::styled_button("Get", Message::Get),
					space::horizontal().width(20),
					components::styled_button("Set", Message::Set),
					space::horizontal().width(20),
					components::styled_button("Save", Message::Save),
					space::horizontal(),
				]
				.padding(16)
				.align_y(Center);

				let status_bar = container(center(
					row![
						text("> "),
						text(status.clone()),
						space::horizontal(),
						text(" <")
					]
					.spacing(1),
				))
				.height(30)
				.padding(1)
				.width(Fill);

				column![
					components::title_bar(),
					header,
					main_content,
					button_bar,
					status_bar
				]
				.into()
			}
		}
	}
}
