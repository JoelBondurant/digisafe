use crate::storage::database::Database;
use iced::widget::text_editor;

#[derive(Clone)]
pub enum Message {
	AttemptUnlock,
	ClearClipboard,
	CloseWindow,
	CopyPassword,
	DbNameChanged(String),
	DbPasswordChanged(String),
	DragWindow,
	Lock,
	PasswordEntryGet,
	PasswordEntryNameInput(String),
	PasswordEntryNoteAction(text_editor::Action),
	PasswordEntryPasswordInput(String),
	PasswordEntrySet,
	PasswordEntryTagsInput(String),
	PasswordEntryUrlInput(String),
	PasswordEntryUsernameInput(String),
	QueryInput(String),
	QuerySubmit,
	Save,
	SaveResult(String),
	Tick,
	TogglePasswordVisibility,
	UnlockResult(Database),
}
