use crate::storage::atlas::FieldAtlas;
use std::time::SystemTime;
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
	Pin = 4u8,
	Note = 5u8,
	Url = 6u8,
	Tags = 7u8,
	CreatedTimestamp = 8u8,
	ModifiedTimestamp = 9u8,
}

#[derive(Clone)]
pub struct PasswordEntry {
	field_atlas: FieldAtlas,
}

impl From<FieldAtlas> for PasswordEntry {
	fn from(field_atlas: FieldAtlas) -> Self {
		PasswordEntry { field_atlas }
	}
}

impl PasswordEntry {
	pub fn new() -> Self {
		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.to_string()
			.as_bytes()
			.to_vec();
		let mut field_atlas = FieldAtlas::default();
		field_atlas.set(PasswordField::CreatedTimestamp as u8, now.clone());
		field_atlas.set(PasswordField::ModifiedTimestamp as u8, now);
		PasswordEntry { field_atlas }
	}
	pub fn set_name(&mut self, name: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Name as u8, name.as_bytes().to_vec());
	}
	pub fn set_username(&mut self, username: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Username as u8, username.as_bytes().to_vec());
	}
	pub fn set_password(&mut self, password: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Password as u8, password.as_bytes().to_vec());
	}
	pub fn set_pin(&mut self, pin: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Pin as u8, pin.as_bytes().to_vec());
	}
	pub fn set_url(&mut self, url: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Url as u8, url.as_bytes().to_vec());
	}
	pub fn set_tags(&mut self, tags: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Tags as u8, tags.as_bytes().to_vec());
	}
	pub fn set_note(&mut self, note: &str) {
		self.update_ts();
		self.field_atlas
			.set(PasswordField::Note as u8, note.as_bytes().to_vec());
	}
	fn update_ts(&mut self) {
		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.to_string()
			.as_bytes()
			.to_vec();
		self.field_atlas
			.set(PasswordField::ModifiedTimestamp as u8, now);
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
	pub fn get_pin(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Pin as u8)
			.unwrap_or_default()
	}
	pub fn get_url(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Url as u8)
			.unwrap_or_default()
	}
	pub fn get_tags(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Tags as u8)
			.unwrap_or_default()
	}
	pub fn get_note(&self) -> &str {
		self.field_atlas
			.get_str(PasswordField::Note as u8)
			.unwrap_or_default()
	}
	pub fn get_timestamps(&self) -> String {
		use chrono::{DateTime, Local, TimeZone, Utc};
		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.to_string();
		let mts = self
			.field_atlas
			.get_str(PasswordField::ModifiedTimestamp as u8)
			.unwrap_or(&now)
			.parse::<i64>()
			.unwrap();
		let cts = self
			.field_atlas
			.get_str(PasswordField::CreatedTimestamp as u8)
			.unwrap_or(&now)
			.parse::<i64>()
			.unwrap();
		let mdt_utc = Utc.timestamp_opt(mts, 0).unwrap();
		let cdt_utc = Utc.timestamp_opt(cts, 0).unwrap();
		let mdt_local = DateTime::<Local>::from(mdt_utc);
		let cdt_local = DateTime::<Local>::from(cdt_utc);
		let mdt = mdt_local.format("%Y-%m-%d %H:%M:%S");
		let cdt = cdt_local.format("%Y-%m-%d %H:%M:%S");
		format!("Modified: {mdt}, Created: {cdt} ")
	}
	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		self.field_atlas.serialize()
	}
}
