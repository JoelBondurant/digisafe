use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub enum Message {
	AttemptUnlock,
	UnlockResult([u8; 32]),
	DbNameChanged(String),
	PasswordChanged(String),
	QueryInput(String),
	QuerySubmit,
	ValueAction(text_editor::Action),
	Get,
	Set,
	Save,
	CloseWindow,
	DragWindow,
}
