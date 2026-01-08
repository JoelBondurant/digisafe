use iced::widget::{center, column, container, text, text_editor, text_input};
use iced::{Color, Element, Fill, Length, Task, Theme};
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
	_db: Database,
}

#[derive(Debug, Clone)]
enum Message {
	QueryInput(String),
	QuerySubmit,
	ValueAction(text_editor::Action),
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
				println!("QuerySubmit: {}", self.query);
			}
			Message::ValueAction(action) => self.value.perform(action),
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

		let main_content = container(center(
			column![value_editor, text(format!("Filtering for: {}", self.query)),].spacing(20),
		))
		.padding(1)
		.width(Length::Fill);

		column![header, main_content].into()
	}

	fn title(&self) -> String {
		State::NAME.to_string()
	}
}
