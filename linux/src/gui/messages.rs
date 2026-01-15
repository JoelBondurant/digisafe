use crate::storage::volatile::Database;
use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
	AttemptUnlock,
	UnlockResult(Database),
	DbNameChanged(String),
	PasswordChanged(String),
	QueryInput(String),
	QuerySubmit,
	ValueAction(text_editor::Action),
	Get,
	Set,
	Save,
	SaveResult(String),
	CloseWindow,
	DragWindow,
}
