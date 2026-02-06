use crate::gui::messages::Message;
use crate::{gui::colors, storage::entry::PasswordEntry};
use iced::{
	advanced::text::highlighter::PlainText,
	border, font,
	theme::{Palette, Theme},
	widget::{
		button, center, column, container, mouse_area, row, space, text, text_editor, text_input,
		tooltip, TextEditor, TextInput, Tooltip,
	},
	Alignment, Background, Center, Color, Element, Fill, Font,
};

const DEFAULT_BUTTON_SIZE: (u32, u32) = (120, 40);

pub fn theme() -> Theme {
	Theme::custom(
		"BlackHole".to_string(),
		Palette {
			background: colors::BG_PRIMARY,
			danger: colors::DANGER,
			primary: colors::PRIMARY,
			success: colors::SUCCESS,
			text: colors::TEXT_PRIMARY,
			warning: colors::WARNING,
		},
	)
}

pub fn title_bar<'a>() -> Element<'a, Message> {
	container(
		row![
			mouse_area(container(row![
				space::horizontal(),
				space::horizontal().width(42),
				text("DigiSafe")
					.size(16)
					.font(Font {
						weight: font::Weight::Bold,
						..Default::default()
					})
					.color(colors::BRAND_GREEN),
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
						background: Some(Background::Color(colors::BRAND_PURPLE)),
						text_color: colors::TEXT_TITLE_BUTTON_HOVER,
						..button::Style::default()
					},
					_ => button::Style {
						background: Some(Background::Color(Color::TRANSPARENT)),
						text_color: colors::TEXT_TITLE_BUTTON,
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
		background: Some(colors::BG_SECONDARY.into()),
		border: border::Border {
			color: colors::BORDER_SECONDARY,
			width: 2.0,
			radius: 0.0.into(),
		},
		..Default::default()
	})
	.into()
}

pub fn unlock_screen<'a>(
	db_name: &str,
	db_password: &str,
	is_processing: &bool,
) -> Element<'a, Message> {
	let unlock_panel = container(center(
		column![
			center(text("Unlock Database").color(colors::BRAND_GREEN).size(18)).height(30),
			styled_text_input("db_name...", db_name).on_input(Message::DbNameChanged),
			styled_text_input("db_password...", db_password)
				.on_input(Message::DbPasswordChanged)
				.secure(true)
				.on_input(Message::DbPasswordChanged)
				.on_submit(Message::AttemptUnlock),
			center(styled_button(
				if *is_processing {
					"Unlocking"
				} else {
					"Unlock"
				},
				Message::AttemptUnlock,
				DEFAULT_BUTTON_SIZE,
			))
			.height(40),
			space().height(100),
		]
		.spacing(20)
		.width(600),
	))
	.width(Fill)
	.height(Fill);

	column![title_bar(), unlock_panel].into()
}

fn styled_tooltip<'a, Message>(
	underlay: impl Into<Element<'a, Message>>,
	label: &'a str,
) -> Tooltip<'a, Message>
where
	Message: 'a,
{
	tooltip(
		underlay,
		container(text(label.to_string()).color(colors::BRAND_GREEN).size(18)).padding(10),
		tooltip::Position::Right,
	)
}

pub fn password_screen<'a>(
	query: &str,
	password_entry: &PasswordEntry,
	is_password_visible: &bool,
	note: &'a text_editor::Content,
	status: &'a str,
) -> Element<'a, Message> {
	let query_input = styled_tooltip(
		styled_query_input("query...", query)
			.on_input(Message::QueryInput)
			.on_submit(Message::QuerySubmit),
		"Query  ",
	);
	let header = container(column![query_input]).padding(8).width(Fill);
	let name_input = styled_tooltip(
		styled_text_input("name", password_entry.get_name())
			.on_input(Message::PasswordEntryNameInput)
			.on_submit(Message::PasswordEntrySet),
		"Name  ",
	);
	let username_input = styled_tooltip(
		styled_text_input("username", password_entry.get_username())
			.on_input(Message::PasswordEntryUsernameInput)
			.on_submit(Message::PasswordEntrySet),
		"Username  ",
	);
	let password_input = styled_tooltip(
		styled_text_input("password", password_entry.get_password())
			.secure(!is_password_visible)
			.on_input(Message::PasswordEntryPasswordInput)
			.on_submit(Message::PasswordEntrySet),
		"Password                    ",
	);
	let mini_button_size = (44, 44);
	let password_copy_button = styled_button("📋", Message::CopyPassword, mini_button_size);
	let password_visibility_toggle = styled_button(
		if *is_password_visible { "👀" } else { "👁" },
		Message::TogglePasswordVisibility,
		mini_button_size,
	);
	let url_input = styled_tooltip(
		styled_text_input("url", password_entry.get_url())
			.on_input(Message::PasswordEntryUrlInput)
			.on_submit(Message::PasswordEntrySet),
		"URL  ",
	);
	let tags_input = styled_tooltip(
		styled_text_input("tags", password_entry.get_tags())
			.on_input(Message::PasswordEntryTagsInput)
			.on_submit(Message::PasswordEntrySet),
		"Tags  ",
	);
	let note_editor = styled_tooltip(
		styled_text_editor("note".into(), note).on_action(Message::PasswordEntryNoteAction),
		"Note  ",
	);
	let main_content = container(center(
		column![
			name_input,
			username_input,
			row![
				password_input,
				password_copy_button,
				password_visibility_toggle
			],
			url_input,
			tags_input,
			note_editor,
		]
		.spacing(4),
	))
	.padding(4)
	.width(Fill);
	let button_bar = row![
		space::horizontal(),
		styled_button("Get", Message::PasswordEntryGet, DEFAULT_BUTTON_SIZE),
		space::horizontal().width(20),
		styled_button("Set", Message::PasswordEntrySet, DEFAULT_BUTTON_SIZE),
		space::horizontal().width(20),
		styled_button("Save", Message::Save, DEFAULT_BUTTON_SIZE),
		space::horizontal(),
	]
	.padding(16)
	.align_y(Center);
	let status_bar = container(center(
		row![
			text("> ").color(colors::BRAND_PURPLE),
			text(status).color(colors::TEXT_STATUS),
			space::horizontal(),
			text(" <").color(colors::BRAND_PURPLE)
		]
		.spacing(1),
	))
	.height(30)
	.padding(1)
	.width(Fill);
	column![title_bar(), header, main_content, button_bar, status_bar].into()
}

pub fn styled_button<'a, Message: Clone + 'a>(
	label: &str,
	msg: Message,
	size: (u32, u32),
) -> Element<'a, Message> {
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
	.width(size.0)
	.height(size.1)
	.style(|theme: &Theme, status: button::Status| {
		let base = button::primary(theme, status);
		match status {
			button::Status::Hovered => button::Style {
				background: Some(Background::Color(colors::BG_BUTTON_HOVER)),
				border: border::Border {
					color: colors::BORDER_ACCENT,
					width: 2.0,
					radius: 5.0.into(),
				},
				text_color: colors::TEXT_SECONDARY,
				..base
			},
			_ => button::Style {
				background: Some(Background::Color(colors::BG_BUTTON)),
				border: border::Border {
					color: colors::BORDER_PRIMARY,
					width: 1.0,
					radius: 5.0.into(),
				},
				text_color: colors::TEXT_SECONDARY,
				..base
			},
		}
	})
	.on_press(msg)
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
				background: Background::Color(colors::BG_INPUT_FOCUS),
				border: border::Border {
					color: colors::BORDER_ACCENT,
					width: 2.0,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER_HOVER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			text_input::Status::Hovered => text_input::Style {
				background: Background::Color(colors::BG_INPUT_HOVER),
				border: border::Border {
					color: colors::BORDER_HOVER,
					width: 1.5,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			_ => text_input::Style {
				background: Background::Color(colors::BG_INPUT),
				border: border::Border {
					color: colors::BORDER_PRIMARY,
					width: 1.0,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
		})
}

pub fn styled_query_input<'a, Message: Clone + 'a>(
	default_str: &str,
	input_str: &str,
) -> TextInput<'a, Message> {
	text_input(default_str, input_str)
		.padding(10)
		.size(18)
		.style(|_theme: &Theme, status: text_input::Status| match status {
			text_input::Status::Focused { .. } => text_input::Style {
				background: Background::Color(colors::BG_INPUT_FOCUS),
				border: border::Border {
					color: colors::BORDER_ACCENT_QUERY,
					width: 2.0,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER_HOVER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			text_input::Status::Hovered => text_input::Style {
				background: Background::Color(colors::BG_INPUT_HOVER),
				border: border::Border {
					color: colors::BORDER_HOVER_QUERY,
					width: 1.5,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			_ => text_input::Style {
				background: Background::Color(colors::BG_INPUT),
				border: border::Border {
					color: colors::BORDER_PRIMARY_QUERY,
					width: 1.0,
					radius: 5.0.into(),
				},
				icon: colors::TEXT_SECONDARY,
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
		})
}

pub fn styled_text_editor<'a>(
	id: String,
	content: &'a text_editor::Content,
) -> TextEditor<'a, PlainText, Message> {
	text_editor(content)
		.id(id)
		.size(18)
		.height(Fill)
		.wrapping(text::Wrapping::Word)
		.style(|_theme: &Theme, status: text_editor::Status| match status {
			text_editor::Status::Focused { .. } => text_editor::Style {
				background: Background::Color(colors::BG_INPUT_FOCUS),
				border: border::Border {
					color: colors::BORDER_ACCENT,
					width: 2.0,
					radius: 5.0.into(),
				},
				placeholder: colors::TEXT_PLACEHOLDER_HOVER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			text_editor::Status::Hovered => text_editor::Style {
				background: Background::Color(colors::BG_INPUT_HOVER),
				border: border::Border {
					color: colors::BORDER_HOVER,
					width: 1.5,
					radius: 5.0.into(),
				},
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
			_ => text_editor::Style {
				background: Background::Color(colors::BG_INPUT),
				border: border::Border {
					color: colors::BORDER_PRIMARY,
					width: 1.0,
					radius: 5.0.into(),
				},
				placeholder: colors::TEXT_PLACEHOLDER,
				value: colors::TEXT_SECONDARY,
				selection: colors::SELECTION,
			},
		})
}
