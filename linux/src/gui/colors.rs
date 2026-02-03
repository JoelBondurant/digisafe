use iced::Color;

pub const BG_PRIMARY: Color = rgb(1, 1, 1);
pub const BG_SECONDARY: Color = rgb(4, 4, 4);
pub const BG_INPUT: Color = rgb(4, 4, 8);
pub const BG_INPUT_HOVER: Color = rgb(10, 8, 16);
pub const BG_INPUT_FOCUS: Color = rgb(8, 4, 10);
pub const BG_BUTTON: Color = rgb(8, 8, 16);
pub const BG_BUTTON_HOVER: Color = rgb(16, 16, 32);

pub const BORDER_PRIMARY: Color = rgb(60, 8, 100);
pub const BORDER_SECONDARY: Color = rgb(32, 80, 32);
pub const BORDER_ACCENT: Color = rgb(110, 10, 240);
pub const BORDER_HOVER: Color = rgb(80, 8, 140);

pub const BORDER_PRIMARY_QUERY: Color = rgb(80, 140, 80);
pub const BORDER_ACCENT_QUERY: Color = rgb(80, 180, 80);
pub const BORDER_HOVER_QUERY: Color = rgb(60, 140, 60);

pub const TEXT_PRIMARY: Color = rgb(230, 230, 230);
pub const TEXT_SECONDARY: Color = rgb(200, 180, 200);
pub const TEXT_PLACEHOLDER: Color = rgb(80, 70, 80);
pub const TEXT_PLACEHOLDER_HOVER: Color = rgb(100, 90, 100);
pub const TEXT_TITLE_BUTTON: Color = rgb(120, 120, 120);
pub const TEXT_TITLE_BUTTON_HOVER: Color = rgb(100, 250, 100);

pub const BRAND_GREEN: Color = rgb(80, 180, 80);
pub const BRAND_PURPLE: Color = rgb(150, 4, 250);

pub const DANGER: Color = rgb(200, 40, 40);
pub const PRIMARY: Color = rgb(100, 100, 255);
pub const SELECTION: Color = rgb(110, 10, 240);
pub const SUCCESS: Color = rgb(40, 200, 40);
pub const WARNING: Color = rgb(200, 80, 80);

const fn rgb(r: u8, g: u8, b: u8) -> Color {
	Color::from_rgb8(r, g, b)
}
