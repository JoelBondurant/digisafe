use crate::storage::database::Database;
use iced::widget::text_editor;

#[derive(Clone)]
pub enum Message {
	AttemptUnlock,
	UnlockResult(Database),
	DbNameChanged(String),
	DbPasswordChanged(String),
	QueryInput(String),
	QuerySubmit,
	PasswordEntryNameInput(String),
	PasswordEntryUsernameInput(String),
	PasswordEntryPasswordInput(String),
	PasswordEntryNoteAction(text_editor::Action),
	PasswordEntryGet,
	PasswordEntrySet,
	Save,
	SaveResult(String),
	CloseWindow,
	DragWindow,
}
