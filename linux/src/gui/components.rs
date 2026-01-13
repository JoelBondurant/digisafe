use iced::{
	border, font,
	theme::{Palette, Theme},
	widget::{button, text},
	Alignment, Background, Color, Element, Fill, Font,
};

pub fn my_button<'a, Message: Clone + 'a>(label: String, msg: Message) -> Element<'a, Message> {
	button(
		text(label)
			.size(18)
			.width(Fill)
			.align_x(Alignment::Center)
			.align_y(Alignment::Center)
			.font(Font {
				weight: font::Weight::Semibold,
				..Default::default()
			}),
	)
	.width(100)
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
