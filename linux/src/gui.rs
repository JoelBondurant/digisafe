use crate::storage::Database;
use iced::{
	border, font,
	theme::{Palette, Theme},
	widget::{
		button, center, column, container, mouse_area, row, space, text, text_editor, text_input,
	},
	window, Alignment, Background, Center, Color, Element, Fill, Font, Task,
};

#[derive(Default)]
struct State {
	query: String,
	value: text_editor::Content,
	status: String,
	db: Database,
}

#[derive(Debug, Clone)]
enum Message {
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
		.theme(State::theme)
		.title(State::title)
		.window(iced::window::Settings {
			decorations: false,
			transparent: false,
			..Default::default()
		})
		.run()
}

fn my_button<'a, Message: Clone + 'a>(label: String, msg: Message) -> Element<'a, Message> {
	button(
		text(label)
			.size(20)
			.width(Fill)
			.align_x(Alignment::Center)
			.align_y(Alignment::Center)
			.font(Font {
				weight: font::Weight::Semibold,
				..Default::default()
			}),
	)
	.width(80)
	.style(|theme: &Theme, status: button::Status| {
		let base = button::primary(theme, status);
		match status {
			button::Status::Hovered => button::Style {
				background: Some(Background::Color(Color::from_rgb8(16, 16, 32))),
				border: border::Border {
					color: Color::from_rgb8(110, 10, 240),
					width: 2.0,
					radius: 5.0.into(),
				},
				text_color: Color::from_rgb8(20, 250, 20),
				..base
			},
			_ => button::Style {
				background: Some(Background::Color(Color::from_rgb8(8, 8, 16))),
				border: border::Border {
					color: Color::from_rgb8(60, 8, 100),
					width: 1.0,
					radius: 5.0.into(),
				},
				text_color: Color::from_rgb8(200, 180, 200),
				..base
			},
		}
	})
	.on_press(msg)
	.into()
}

impl State {
	const NAME: &str = "DigiSafe";

	fn new() -> Self {
		Self::default()
	}

	fn theme(_state: &State) -> Theme {
		black_hole_theme()
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::QueryInput(new_text) => {
				self.query = new_text;
			}
			Message::QuerySubmit => {
				return Task::done(Message::Get);
			}
			Message::ValueAction(action) => self.value.perform(action),
			Message::Get => {
				if let Some(found_value) = self.db.get(&self.query) {
					self.status = "Entry retrieved.".to_string();
					self.value = text_editor::Content::with_text(&found_value);
				} else {
					self.status = "Entry not retrieved.".to_string();
					self.value = text_editor::Content::new();
				}
			}
			Message::Set => {
				let content_string = self.value.text();
				if !self.query.is_empty() {
					self.db.set(self.query.clone(), content_string);
					self.status = "Entry set.".to_string();
				} else {
					self.status = "Error: Query was empty".to_owned();
				}
			}
			Message::Save => {
				self.status = "Save database not yet implemented.".to_owned();
			}
			Message::CloseWindow => {
				return window::latest().and_then(window::close);
			}
			Message::DragWindow => {
				return window::latest().and_then(window::drag);
			}
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
				button(text("✕").size(16).align_y(Center).align_x(Center))
					.width(40)
					.style(|_theme: &Theme, status: button::Status| {
						match status {
							button::Status::Hovered => button::Style {
								background: Some(Background::Color(Color::from_rgb8(200, 40, 40))),
								text_color: Color::WHITE,
								..button::Style::default()
							},
							_ => button::Style {
								background: Some(Background::Color(Color::TRANSPARENT)),
								text_color: Color::from_rgb8(150, 150, 150),
								..button::Style::default()
							},
						}
					})
					.on_press(Message::CloseWindow),
			]
			.padding(8)
			.align_y(iced::Center),
		)
		.width(Fill)
		.style(|_theme| container::Style {
			background: Some(Color::from_rgb8(10, 10, 10).into()),
			border: border::Border {
				color: Color::from_rgb8(40, 240, 40),
				width: 0.0,
				radius: 0.0.into(),
			},
			..Default::default()
		});

		let query_bar = text_input("Search passwords...", &self.query)
			.on_input(Message::QueryInput)
			.on_submit(Message::QuerySubmit)
			.padding(10)
			.size(18)
			.style(|_theme: &Theme, status: text_input::Status| match status {
				text_input::Status::Focused { .. } => text_input::Style {
					background: Background::Color(Color::from_rgb8(16, 16, 32)),
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
					background: Background::Color(Color::from_rgb8(12, 12, 24)),
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
					background: Background::Color(Color::from_rgb8(8, 8, 16)),
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

		let value_editor = text_editor(&self.value)
			.id("value")
			.size(18)
			.height(Fill)
			.on_action(Message::ValueAction)
			.wrapping(text::Wrapping::Word)
			.style(|_theme: &Theme, status: text_editor::Status| match status {
				text_editor::Status::Focused { .. } => text_editor::Style {
					background: Background::Color(Color::from_rgb8(16, 16, 32)),
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
					background: Background::Color(Color::from_rgb8(12, 12, 24)),
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
					background: Background::Color(Color::from_rgb8(8, 8, 16)),
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
			my_button("Get".into(), Message::Get),
			space::horizontal().width(20),
			my_button("Set".into(), Message::Set),
			space::horizontal().width(20),
			my_button("Save".into(), Message::Save),
			space::horizontal(),
		]
		.padding(10)
		.align_y(Center);

		let status_bar = container(center(
			row![
				text("> "),
				text(self.status.clone()),
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

	fn title(&self) -> String {
		State::NAME.to_string()
	}
}

fn black_hole_theme() -> Theme {
	Theme::custom(
		"BlackHole".to_string(),
		Palette {
			background: Color::from_rgb8(1, 1, 1),
			danger: Color::from_rgb8(200, 40, 40),
			primary: Color::from_rgb8(100, 100, 255),
			success: Color::from_rgb8(40, 200, 40),
			text: Color::from_rgb8(230, 230, 230),
			warning: Color::from_rgb8(200, 80, 80),
		},
	)
}
