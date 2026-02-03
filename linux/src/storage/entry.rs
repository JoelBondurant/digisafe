use crate::storage::atlas::FieldAtlas;
use zeroize::Zeroizing;

#[derive(Debug)]
#[repr(u8)]
pub enum MetaField {
	Name = 1u8,
	Value = 2u8,
}

#[derive(Default)]
pub struct MetaEntry {
	field_atlas: FieldAtlas,
}

impl From<FieldAtlas> for MetaEntry {
	fn from(field_atlas: FieldAtlas) -> Self {
		MetaEntry { field_atlas }
	}
}

impl MetaEntry {
	pub fn new(name: &str, value: &str) -> Self {
		let mut meta_entry = MetaEntry::default();
		meta_entry.set_name(name);
		meta_entry.set_value(value);
		meta_entry
	}
	pub fn set_name(&mut self, name: &str) {
		self.field_atlas
			.set(MetaField::Name as u8, name.as_bytes().to_vec());
	}
	pub fn set_value(&mut self, value: &str) {
		self.field_atlas
			.set(MetaField::Value as u8, value.as_bytes().to_vec());
	}
	pub fn get_name(&self) -> &str {
		self.field_atlas.get_str(MetaField::Name as u8).unwrap()
	}
	pub fn get_value(&self) -> &str {
		self.field_atlas.get_str(MetaField::Value as u8).unwrap()
	}
	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		self.field_atlas.serialize()
	}
}

#[derive(Debug)]
#[repr(u8)]
pub enum PasswordField {
	Name = 1u8,
	Username = 2u8,
	Password = 3u8,
	Note = 4u8,
	//	Url = 5u8,
	//	Tags = 6u8,
	//	CreatedTimestamp = 7u8,
	//	ModifiedTimestamp = 8u8,
}

#[derive(Clone, Default)]
pub struct PasswordEntry {
	field_atlas: FieldAtlas,
}

impl From<FieldAtlas> for PasswordEntry {
	fn from(field_atlas: FieldAtlas) -> Self {
		PasswordEntry { field_atlas }
	}
}

impl PasswordEntry {
	pub fn set_name(&mut self, name: &str) {
		self.field_atlas
			.set(PasswordField::Name as u8, name.as_bytes().to_vec());
	}
	pub fn set_username(&mut self, username: &str) {
		self.field_atlas
			.set(PasswordField::Username as u8, username.as_bytes().to_vec());
	}
	pub fn set_password(&mut self, password: &str) {
		self.field_atlas
			.set(PasswordField::Password as u8, password.as_bytes().to_vec());
	}
	pub fn set_note(&mut self, note: &str) {
		self.field_atlas
			.set(PasswordField::Note as u8, note.as_bytes().to_vec());
	}
	pub fn get_name(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Name as u8)
			.unwrap_or_default()
	}
	pub fn get_username(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Username as u8)
			.unwrap_or_default()
	}
	pub fn get_password(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Password as u8)
			.unwrap_or_default()
	}
	pub fn get_note(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Note as u8)
			.unwrap_or_default()
	}
	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		self.field_atlas.serialize()
	}
}
