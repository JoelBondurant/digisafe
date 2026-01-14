use crate::gui::messages::Message;

use iced::widget::text_input;
use iced::widget::text_input::TextInput;
use iced::{
	border, font,
	theme::{Palette, Theme},
	widget::{button, container, mouse_area, row, space, text},
	Alignment, Background, Center, Color, Element, Fill, Font,
};

pub fn styled_button<'a, Message: Clone + 'a>(label: &str, msg: Message) -> Element<'a, Message> {
	button(
		text(label.to_string())
			.size(18)
			.width(Fill)
			.align_x(Alignment::Center)
			.align_y(Alignment::Center)
			.font(Font {
				weight: font::Weight::Semibold,
				..Default::default()
			}),
	)
	.width(120)
	.height(40)
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

pub fn black_hole_theme() -> Theme {
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

pub fn title_bar<'a>() -> Element<'a, Message> {
	container(
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
	})
	.into()
}

pub fn styled_text_input<'a, Message: Clone + 'a>(
	default_str: &str,
	input_str: &str,
) -> TextInput<'a, Message> {
	text_input(default_str, input_str)
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
		})
}

/*
use iced::advanced::text::highlighter::Highlighter;
use iced::widget::text_editor;
use iced::widget::text_editor::TextEditor;
pub fn styled_text_editor<'a>(
	content: &'a text_editor::Content,
) -> TextEditor<'a, Highlighter, Theme> {
	text_editor(content)
		.id("value")
		.size(18)
		.height(Fill)
		.wrapping(text::Wrapping::Word)
		.highlight(
			Highlighter::default(),
			|highlighter: &mut Highlighter, theme: &Theme| highlighter.to_format(theme),
		)
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
		})
}
*/
