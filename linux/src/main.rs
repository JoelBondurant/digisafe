use iced::widget::{button, center, column, container, row, space, text, text_editor, text_input};
use iced::{
	border, font, Alignment, Background, Center, Color, Element, Fill, Font, Length, Task, Theme,
};
use std::collections::BTreeMap;

pub fn main() -> iced::Result {
	iced::application(State::new, State::update, State::view)
		.theme(State::theme)
		.title(State::title)
		.run()
}

#[derive(Default)]
struct Database {
	_map: BTreeMap<String, String>,
}

#[derive(Default)]
struct State {
	query: String,
	value: text_editor::Content,
	status: String,
	_db: Database,
}

#[derive(Debug, Clone)]
enum Message {
	QueryInput(String),
	QuerySubmit,
	ValueAction(text_editor::Action),
	Get,
	Set,
	Save,
}

fn my_button<'a, Message: Clone + 'a>(label: String, msg: Message) -> Element<'a, Message> {
	button(
		text(label)
			.size(20)
			.width(Length::Fill)
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
				background: Some(Background::Color(Color::from_rgb8(52, 54, 76))),
				border: border::Border {
					color: Color::from_rgb8(40, 240, 40),
					width: 2.0,
					radius: 5.0.into(),
				},
				text_color: Color::from_rgb8(200, 255, 200),
				..base
			},
			_ => button::Style {
				background: Some(Background::Color(Color::from_rgb8(26, 27, 38))),
				border: border::Border {
					color: Color::from_rgb8(10, 80, 10),
					width: 1.0,
					radius: 5.0.into(),
				},
				text_color: Color::from_rgb8(102, 109, 138),
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
		Theme::TokyoNight
	}

	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::QueryInput(new_text) => {
				self.query = new_text;
			}
			Message::QuerySubmit => {
				self.status = "Query submitted.".to_owned();
			}
			Message::ValueAction(action) => {
				self.status = format!("Modify entry: {}", self.query);
				self.value.perform(action)
			}
			Message::Get => {
				self.status = format!("Get entry: {}", self.query);
			}
			Message::Set => {
				self.status = format!("Set entry: {}", self.query);
			}
			Message::Save => {
				self.status = "Save database.".to_owned();
			}
		}
		Task::none()
	}

	fn view(&self) -> Element<'_, Message> {
		let query_bar = text_input("Search passwords...", &self.query)
			.on_input(Message::QueryInput)
			.on_submit(Message::QuerySubmit)
			.padding(10)
			.size(18);

		let header = container(query_bar)
			.padding(1)
			.width(Length::Fill)
			.style(|_theme| container::Style {
				background: Some(Color::from_rgb8(40, 240, 40).into()),
				..Default::default()
			});

		let value_editor = text_editor(&self.value)
			.id("value")
			.height(Fill)
			.on_action(Message::ValueAction)
			.wrapping(text::Wrapping::Word);

		let main_content = container(center(column![value_editor].spacing(20)))
			.padding(1)
			.width(Length::Fill);

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
				text(">"),
				text(self.status.clone()),
				space::horizontal(),
				text("<")
			]
			.spacing(1),
		))
		.height(30)
		.padding(1)
		.width(Length::Fill);

		column![header, main_content, button_bar, status_bar].into()
	}

	fn title(&self) -> String {
		State::NAME.to_string()
	}
}
