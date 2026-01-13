pub mod components;

use crate::crypto;
use crate::storage::volatile::Database;
use iced::{
	border, font,
	theme::Theme,
	widget::{
		button, center, column, container, mouse_area, row, space, text, text_editor, text_input,
	},
	window, Background, Center, Color, Element, Fill, Font, Size, Task,
};
use std::{
	mem,
	sync::{Arc, RwLock},
};

enum AppState {
	Locked {
		db_name: String,
		password: String,
		is_processing: bool,
	},
	Unlocked {
		db_name: String,
		master_key: [u8; 32],
		query: String,
		value: text_editor::Content,
		status: String,
		db: Arc<RwLock<Database>>,
	},
}

struct State {
	app_state: AppState,
}

#[derive(Debug, Clone)]
enum Message {
	AttemptUnlock,
	UnlockResult([u8; 32]),
	DbNameChanged(String),
	PasswordChanged(String),
	QueryInput(String),
	QuerySubmit,
	ValueAction(text_editor::Action),
	Get,
	Set,
	Save,
	CloseWindow,
	DragWindow,
}

pub type Result = iced::Result;

pub fn run() -> Result {
	iced::application(State::new, State::update, State::view)
		.theme(components::black_hole_theme())
		.title(State::title)
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
	const NAME: &str = "DigiSafe";

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
		match mem::replace(
			&mut self.app_state,
			AppState::Locked {
				db_name: "".into(),
				password: "".into(),
				is_processing: false,
			},
		) {
			AppState::Locked {
				db_name,
				password,
				is_processing,
			} => match message {
				Message::AttemptUnlock => {
					self.app_state = AppState::Locked {
						db_name: db_name.clone(),
						password: password.clone(),
						is_processing: true,
					};
					let pw = password.clone();
					return Task::perform(
						async move {
							let salt = b"digisafe";
							crypto::master_key_derivation(pw.as_bytes(), salt)
						},
						Message::UnlockResult,
					);
				}
				Message::UnlockResult(master_key) => {
					self.app_state = AppState::Unlocked {
						db_name: db_name.clone(),
						master_key,
						query: "".into(),
						value: text_editor::Content::new(),
						status: "Unlocked".into(),
						db: Arc::new(RwLock::new(Database::default())),
					};
				}
				Message::DbNameChanged(new_db_name) => {
					self.app_state = AppState::Locked {
						db_name: new_db_name,
						password: password.clone(),
						is_processing,
					};
				}
				Message::PasswordChanged(new_password) => {
					self.app_state = AppState::Locked {
						db_name: db_name.clone(),
						password: new_password,
						is_processing,
					};
				}
				Message::CloseWindow => {
					return window::latest().and_then(window::close);
				}
				Message::DragWindow => {
					self.app_state = AppState::Locked {
						db_name: db_name.clone(),
						password: password.clone(),
						is_processing,
					};
					return window::latest().and_then(window::drag);
				}
				_ => {}
			},
			AppState::Unlocked {
				db_name,
				master_key,
				query,
				value,
				status,
				db,
			} => match message {
				Message::QueryInput(new_text) => {
					self.app_state = AppState::Unlocked {
						db_name: db_name.clone(),
						master_key,
						query: new_text,
						value: value.clone(),
						status: status.clone(),
						db: db.clone(),
					};
				}
				Message::QuerySubmit => {
					self.app_state = AppState::Unlocked {
						db_name: db_name.clone(),
						master_key,
						query: query.clone(),
						value: value.clone(),
						status: status.clone(),
						db: db.clone(),
					};
					return Task::done(Message::Get);
				}
				Message::ValueAction(action) => {
					let mut new_value = value.clone();
					new_value.perform(action);
					self.app_state = AppState::Unlocked {
						db_name: db_name.clone(),
						master_key,
						query: query.clone(),
						value: new_value,
						status: status.clone(),
						db: db.clone(),
					};
				}
				Message::Get => {
					let aval = db.read().unwrap().get(&query);
					if let Some(found_value) = aval {
						self.app_state = AppState::Unlocked {
							db_name: db_name.clone(),
							master_key,
							query: query.clone(),
							value: text_editor::Content::with_text(&found_value),
							status: "Entry retrieved.".to_string(),
							db: db.clone(),
						};
					} else {
						self.app_state = AppState::Unlocked {
							db_name: db_name.clone(),
							master_key,
							query: query.clone(),
							value: text_editor::Content::new(),
							status: "Entry not retrieved.".to_string(),
							db: db.clone(),
						};
					};
				}
				Message::Set => {
					let content_string = value.text();
					if !query.is_empty() {
						let new_db = db.clone();
						new_db.write().unwrap().set(query.clone(), content_string);
						self.app_state = AppState::Unlocked {
							db_name: db_name.clone(),
							master_key,
							query: query.clone(),
							value: value.clone(),
							status: "Entry set.".to_string(),
							db: new_db,
						};
					} else {
						self.app_state = AppState::Unlocked {
							db_name: db_name.clone(),
							master_key,
							query: query.clone(),
							value: value.clone(),
							status: "Error: Query was empty".to_owned(),
							db: db.clone(),
						};
					}
				}
				Message::Save => {
					self.app_state = AppState::Unlocked {
						db_name: db_name.clone(),
						master_key,
						query: query.clone(),
						value: value.clone(),
						status: "Save database not yet implemented.".to_owned(),
						db: db.clone(),
					};
				}
				Message::CloseWindow => {
					return window::latest().and_then(window::close);
				}
				Message::DragWindow => {
					self.app_state = AppState::Unlocked {
						db_name: db_name.clone(),
						master_key,
						query: query.clone(),
						value: value.clone(),
						status: status.clone(),
						db: db.clone(),
					};
					return window::latest().and_then(window::drag);
				}
				_ => {}
			},
		}
		Task::none()
	}

	fn view(&self) -> Element<'_, Message> {
		let title_bar = container(
			row![
				mouse_area(container(row![
					space::horizontal(),
					text("DigiSafe")
						.size(14)
						.font(Font {
							weight: font::Weight::Bold,
							..Default::default()
						})
						.color(Color::from_rgb8(80, 200, 80)),
					space::horizontal()
				]))
				.on_press(Message::DragWindow),
				button(
					text("✕")
						.font(Font {
							weight: font::Weight::Bold,
							..Default::default()
						})
						.size(18)
						.align_y(Center)
						.align_x(Center)
				)
				.width(42)
				.height(36)
				.style(|_theme: &Theme, status: button::Status| {
					match status {
						button::Status::Hovered => button::Style {
							background: Some(Background::Color(Color::from_rgb8(150, 4, 250))),
							text_color: Color::from_rgb8(100, 250, 100),
							..button::Style::default()
						},
						_ => button::Style {
							background: Some(Background::Color(Color::TRANSPARENT)),
							text_color: Color::from_rgb8(120, 120, 120),
							..button::Style::default()
						},
					}
				})
				.on_press(Message::CloseWindow),
			]
			.padding(0)
			.align_y(iced::Center),
		)
		.width(Fill)
		.height(36)
		.style(|_theme| container::Style {
			background: Some(Color::from_rgb8(4, 4, 4).into()),
			border: border::Border {
				color: Color::from_rgb8(12, 32, 12),
				width: 2.0,
				radius: 0.0.into(),
			},
			..Default::default()
		});
		match &self.app_state {
			AppState::Locked {
				db_name,
				password,
				is_processing,
			} => {
				let unlock_panel = container(center(
					column![
						text("Unlock Database").size(16),
						text_input("name...", db_name)
							.on_input(Message::DbNameChanged)
							.padding(10)
							.size(18)
							.style(|_theme: &Theme, status: text_input::Status| match status {
								text_input::Status::Focused { .. } => text_input::Style {
									background: Background::Color(Color::from_rgb8(8, 4, 10)),
									border: border::Border {
										color: Color::from_rgb8(110, 10, 240),
										width: 2.0,
										radius: 5.0.into(),
									},
									icon: Color::from_rgb8(200, 180, 200),
									placeholder: Color::from_rgb8(100, 90, 100),
									value: Color::from_rgb8(20, 250, 20),
									selection: Color::from_rgb8(110, 10, 240),
								},
								text_input::Status::Hovered => text_input::Style {
									background: Background::Color(Color::from_rgb8(10, 8, 16)),
									border: border::Border {
										color: Color::from_rgb8(80, 8, 140),
										width: 1.5,
										radius: 5.0.into(),
									},
									icon: Color::from_rgb8(200, 180, 200),
									placeholder: Color::from_rgb8(80, 70, 80),
									value: Color::from_rgb8(200, 180, 200),
									selection: Color::from_rgb8(110, 10, 240),
								},
								_ => text_input::Style {
									background: Background::Color(Color::from_rgb8(4, 4, 8)),
									border: border::Border {
										color: Color::from_rgb8(60, 8, 100),
										width: 1.0,
										radius: 5.0.into(),
									},
									icon: Color::from_rgb8(200, 180, 200),
									placeholder: Color::from_rgb8(80, 70, 80),
									value: Color::from_rgb8(200, 180, 200),
									selection: Color::from_rgb8(110, 10, 240),
								},
							}),
						text_input("password...", password)
							.on_input(Message::PasswordChanged)
							.secure(true)
							.on_submit(Message::AttemptUnlock)
							.padding(10)
							.size(18)
							.style(|_theme: &Theme, status: text_input::Status| match status {
								text_input::Status::Focused { .. } => text_input::Style {
									background: Background::Color(Color::from_rgb8(8, 4, 10)),
									border: border::Border {
										color: Color::from_rgb8(110, 10, 240),
										width: 2.0,
										radius: 5.0.into(),
									},
									icon: Color::from_rgb8(200, 180, 200),
									placeholder: Color::from_rgb8(100, 90, 100),
									value: Color::from_rgb8(20, 250, 20),
									selection: Color::from_rgb8(110, 10, 240),
								},
								text_input::Status::Hovered => text_input::Style {
									background: Background::Color(Color::from_rgb8(10, 8, 16)),
									border: border::Border {
										color: Color::from_rgb8(80, 8, 140),
										width: 1.5,
										radius: 5.0.into(),
									},
									icon: Color::from_rgb8(200, 180, 200),
									placeholder: Color::from_rgb8(80, 70, 80),
									value: Color::from_rgb8(200, 180, 200),
									selection: Color::from_rgb8(110, 10, 240),
								},
								_ => text_input::Style {
									background: Background::Color(Color::from_rgb8(4, 4, 8)),
									border: border::Border {
										color: Color::from_rgb8(60, 8, 100),
										width: 1.0,
										radius: 5.0.into(),
									},
									icon: Color::from_rgb8(200, 180, 200),
									placeholder: Color::from_rgb8(80, 70, 80),
									value: Color::from_rgb8(200, 180, 200),
									selection: Color::from_rgb8(110, 10, 240),
								},
							}),
						components::my_button(
							if *is_processing {
								"Unlocking".into()
							} else {
								"Unlock".into()
							},
							Message::AttemptUnlock
						)
					]
					.spacing(20)
					.width(600),
				))
				.width(Fill)
				.height(Fill);

				column![title_bar, unlock_panel].into()
			}
			AppState::Unlocked {
				db_name: _,
				master_key: _,
				query,
				value,
				status,
				db: _,
			} => {
				let query_bar = text_input("search...", query)
					.on_input(Message::QueryInput)
					.on_submit(Message::QuerySubmit)
					.padding(10)
					.size(18)
					.style(|_theme: &Theme, status: text_input::Status| match status {
						text_input::Status::Focused { .. } => text_input::Style {
							background: Background::Color(Color::from_rgb8(8, 4, 10)),
							border: border::Border {
								color: Color::from_rgb8(110, 10, 240),
								width: 2.0,
								radius: 5.0.into(),
							},
							icon: Color::from_rgb8(200, 180, 200),
							placeholder: Color::from_rgb8(100, 90, 100),
							value: Color::from_rgb8(20, 250, 20),
							selection: Color::from_rgb8(110, 10, 240),
						},
						text_input::Status::Hovered => text_input::Style {
							background: Background::Color(Color::from_rgb8(10, 8, 16)),
							border: border::Border {
								color: Color::from_rgb8(80, 8, 140),
								width: 1.5,
								radius: 5.0.into(),
							},
							icon: Color::from_rgb8(200, 180, 200),
							placeholder: Color::from_rgb8(80, 70, 80),
							value: Color::from_rgb8(200, 180, 200),
							selection: Color::from_rgb8(110, 10, 240),
						},
						_ => text_input::Style {
							background: Background::Color(Color::from_rgb8(4, 4, 8)),
							border: border::Border {
								color: Color::from_rgb8(60, 8, 100),
								width: 1.0,
								radius: 5.0.into(),
							},
							icon: Color::from_rgb8(200, 180, 200),
							placeholder: Color::from_rgb8(80, 70, 80),
							value: Color::from_rgb8(200, 180, 200),
							selection: Color::from_rgb8(110, 10, 240),
						},
					});

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
					components::my_button("Get".into(), Message::Get),
					space::horizontal().width(20),
					components::my_button("Set".into(), Message::Set),
					space::horizontal().width(20),
					components::my_button("Save".into(), Message::Save),
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

				column![title_bar, header, main_content, button_bar, status_bar].into()
			}
		}
	}

	fn title(&self) -> String {
		State::NAME.to_string()
	}
}
