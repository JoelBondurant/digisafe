use std::{collections::BTreeMap, mem, sync::{Arc, RwLock}};

use crate::storage::{
	atlas::{EntryAtlas, FieldAtlas},
	entry::{MetaEntry, PasswordEntry}, secret::SecretMemory,
};
use zeroize::{Zeroize, Zeroizing};


#[derive(Debug)]
#[repr(u8)]
pub enum EntryTag {
	Meta = 0u8,
	Password = 1u8,
}

#[derive(Clone)]
pub struct Database {
	db: Arc<RwLock<InteriorDatabase>>,
	pub master_key: Arc<RwLock<SecretMemory>>,
}

impl Database {
	pub fn new(master_key: SecretMemory) -> Self {
		let db = InteriorDatabase::default();
		Self {
			db: Arc::new(RwLock::new(db)),
			master_key: Arc::new(RwLock::new(master_key)),
		}
	}
	pub fn old(master_key: SecretMemory, db: InteriorDatabase) -> Self {
		Self {
			db: Arc::new(RwLock::new(db)),
			master_key: Arc::new(RwLock::new(master_key)),
		}
	}
	pub fn set_password_entry(&self, entry: PasswordEntry) {
		self.db.write().unwrap().set_password_entry(entry);
	}
	pub fn get_password_entry(&self, name: &str) -> Option<PasswordEntry> {
		self.db.read().unwrap().get_password_entry(name)
	}
	pub fn set_meta_entry(&self, entry: MetaEntry) {
		self.db.write().unwrap().set_meta_entry(entry);
	}
	pub fn get_meta_entry(&self, name: &str) -> Option<MetaEntry> {
		self.db.read().unwrap().get_meta_entry(name)
	}
	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		self.db.read().unwrap().serialize()
	}
	pub fn meta_only(&self) -> Self {
		let mut meta = EntryAtlas::default();
		let mut idx = 1u32;
		for (entry_tag, entry_data) in self.db.read().unwrap().entries.entries.values() {
			let entry_tag = unsafe { mem::transmute::<u8, EntryTag>(*entry_tag) };
			if let EntryTag::Meta = entry_tag {
				meta.set(idx, entry_tag as u8, entry_data.to_vec());
				idx += 1;
			}
		}
		Self {
			db: Arc::new(RwLock::new(InteriorDatabase::from_entry_atlas(meta))),
			master_key: Arc::clone(&self.master_key),
		}
	}
	pub fn zeroize(&self) {
		let _ = self.master_key.write().unwrap().zeroize();
		self.db.write().unwrap().zeroize();
	}
}

#[derive(Default)]
pub struct InteriorDatabase {
	next_id: u32,
	entries: EntryAtlas,
	index_by_name: BTreeMap<String, u32>,
}

impl InteriorDatabase {
	fn set_password_entry(&mut self, entry: PasswordEntry) {
		let name = entry.get_name();
		let index_by_name_value = format!("password\x00{}", name);
		let is_new = !self.index_by_name.contains_key(&index_by_name_value);
		let id: u32;
		if is_new {
			id = self.next_id;
			self.next_id += 1;
			self.index_by_name.insert(index_by_name_value, id);
		} else {
			id = *self.index_by_name.get(&index_by_name_value).unwrap();
		}
		self.entries
			.set(id, EntryTag::Password as u8, entry.serialize().to_vec());
	}
	fn get_password_entry(&self, name: &str) -> Option<PasswordEntry> {
		let index_by_name_value = format!("password\x00{}", name);
		if let Some(id) = self.index_by_name.get(&index_by_name_value)
			&& let Some(entry) = self.entries.get(*id) {
			return Some(PasswordEntry::from(FieldAtlas::deserialize(&entry.1)));
		}
		None
	}
	fn set_meta_entry(&mut self, entry: MetaEntry) {
		let name = entry.get_name();
		let index_by_name_value = format!("meta\x00{}", name);
		let is_new = !self.index_by_name.contains_key(&index_by_name_value);
		let id: u32;
		if is_new {
			id = self.next_id;
			self.next_id += 1;
			self.index_by_name.insert(index_by_name_value, id);
		} else {
			id = *self.index_by_name.get(&index_by_name_value).unwrap();
		}
		self.entries
			.set(id, EntryTag::Meta as u8, entry.serialize().to_vec());
	}
	pub fn get_meta_entry(&self, name: &str) -> Option<MetaEntry> {
		let index_by_name_value = format!("meta\x00{}", name);
		if let Some(id) = self.index_by_name.get(&index_by_name_value)
			&& let Some(entry) = self.entries.get(*id) {
			return Some(MetaEntry::from(FieldAtlas::deserialize(&entry.1)));
		}
		None
	}
	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		self.entries.serialize()
	}
	pub fn deserialize(data: &[u8]) -> Self {
		let entry_atlas = EntryAtlas::deserialize(data);
		InteriorDatabase::from_entry_atlas(entry_atlas)
	}
	pub fn from_entry_atlas(entry_atlas: EntryAtlas) -> Self {
		let mut db = InteriorDatabase::default();
		for (entry_tag, entry_data) in entry_atlas.entries.values() {
			let entry = FieldAtlas::deserialize(entry_data);
			let entry_tag = unsafe { mem::transmute::<u8, EntryTag>(*entry_tag) };
			match entry_tag {
				EntryTag::Meta => {
					db.set_meta_entry(MetaEntry::from(entry));
				}
				EntryTag::Password => {
					db.set_password_entry(PasswordEntry::from(entry));
				}
			}
		}
		db
	}
	pub fn zeroize(&mut self) {
		for (_, value) in self.entries.entries.values_mut() {
			value.zeroize();
		}
		let index_by_name = mem::take(&mut self.index_by_name);
		for (mut key, _) in index_by_name {
			key.zeroize();
		}
	}
}
