use std::collections::BTreeMap;
use zeroize::Zeroizing;

#[derive(Clone, Default)]
pub struct FieldAtlas {
	fields: BTreeMap<u8, Zeroizing<Vec<u8>>>,
}

impl FieldAtlas {
	pub fn deserialize(data: &[u8]) -> Self {
		let mut fields = BTreeMap::new();
		let mut cursor = 0;
		while cursor < data.len() {
			let field_type = data[cursor];
			cursor += 1;
			let len_bytes = &data[cursor..cursor + 4];
			let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
			cursor += 4;
			let value = &data[cursor..cursor + len];
			fields.insert(field_type, value.to_vec().into());
			cursor += len;
		}
		FieldAtlas { fields }
	}
	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		let mut buffer = Vec::new();
		for (&field_type, value) in &self.fields {
			buffer.push(field_type);
			let len = value.len() as u32;
			buffer.extend_from_slice(&len.to_le_bytes());
			buffer.extend_from_slice(value.as_ref());
		}
		Zeroizing::new(buffer)
	}
	pub fn get(&self, field_type: u8) -> Option<&[u8]> {
		self.fields.get(&field_type).map(|v| v.as_ref())
	}
	pub fn get_str(&self, field_type: u8) -> Option<&str> {
		self.get(field_type)
			.and_then(|bytes| std::str::from_utf8(bytes).ok())
	}
	pub fn set(&mut self, field_type: u8, value: Vec<u8>) {
		self.fields.insert(field_type, value.into());
	}
}

#[derive(Default)]
pub struct EntryAtlas {
	pub entries: BTreeMap<u32, (u8, Zeroizing<Vec<u8>>)>,
}

impl EntryAtlas {
	pub fn deserialize(data: &[u8]) -> Self {
		let mut entries = BTreeMap::new();
		let mut id = 0u32;
		let mut cursor = 0usize;
		while cursor < data.len() {
			let entry_type = data[cursor];
			cursor += 1;
			let len = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap()) as usize;
			cursor += 4;
			let value = Zeroizing::new(data[cursor..cursor + len].to_vec());
			entries.insert(id, (entry_type, value));
			id += 1;
			cursor += len;
		}
		EntryAtlas { entries }
	}

	pub fn serialize(&self) -> Zeroizing<Vec<u8>> {
		let mut buffer = Vec::new();
		for (entry_type, value) in self.entries.values() {
			buffer.push(*entry_type);
			let len = value.len() as u32;
			buffer.extend_from_slice(&len.to_le_bytes());
			buffer.extend_from_slice(value.as_ref());
		}
		Zeroizing::new(buffer)
	}

	pub fn set(&mut self, id: u32, entry_type: u8, value: Vec<u8>) {
		self.entries.insert(id, (entry_type, Zeroizing::new(value)));
	}

	pub fn get(&self, id: u32) -> Option<(u8, Zeroizing<Vec<u8>>)> {
		self.entries.get(&id).cloned()
	}
}
