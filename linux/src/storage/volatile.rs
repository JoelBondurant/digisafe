#![allow(dead_code)] // iced constructs this by Default.
use memsecurity::{EncryptedMem, ZeroizeBytes};
use std::{
	collections::BTreeMap,
	mem,
	sync::{Arc, RwLock},
};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Debug, Default)]
pub struct Database {
	pub master_key: Arc<RwLock<EncryptedMem>>,
	pub meta: Arc<RwLock<BTreeMap<String, String>>>,
	pub kvmap: Arc<RwLock<BTreeMap<String, EncryptedMem>>>,
}

impl Drop for Database {
	fn drop(&mut self) {
		if let Some(rwlock) = Arc::get_mut(&mut self.kvmap) && let Ok(kvmap) = rwlock.get_mut() {
			for (mut key, _value) in mem::take(kvmap).into_iter() {
				key.zeroize();
			}
		}
	}
}

impl Database {
	pub fn old(
		master_key: [u8; 32],
		meta: Arc<RwLock<BTreeMap<String, String>>>,
		kvmap: Arc<RwLock<BTreeMap<String, String>>>,
	) -> Database {
		let master_key = Zeroizing::new(master_key);
		let encrypted_master_key = Arc::new(RwLock::new(EncryptedMem::new()));
		let _ = encrypted_master_key
			.write()
			.unwrap()
			.encrypt(&master_key)
			.unwrap();
		let kvmap_encrypted = Arc::new(RwLock::new(BTreeMap::<String, EncryptedMem>::new()));
		for (key, value) in kvmap.write().unwrap().iter_mut() {
			let mut encrypted_value = EncryptedMem::new();
			let _ = encrypted_value.encrypt(value);
			kvmap_encrypted
				.write()
				.unwrap()
				.insert(key.to_string(), encrypted_value);
			value.zeroize();
		}
		Database {
			master_key: encrypted_master_key,
			meta,
			kvmap: kvmap_encrypted,
		}
	}

	pub fn new(master_key: [u8; 32], db_name: String) -> Database {
		let master_key = Zeroizing::new(master_key);
		let encrypted_master_key = Arc::new(RwLock::new(EncryptedMem::new()));
		let _ = encrypted_master_key
			.write()
			.unwrap()
			.encrypt(&master_key)
			.unwrap();
		let meta = Arc::new(RwLock::new(BTreeMap::new()));
		meta.write()
			.unwrap()
			.insert("db_name".to_string(), db_name.to_string());
		let kvmap = Arc::new(RwLock::new(BTreeMap::new()));
		Database {
			master_key: encrypted_master_key,
			meta,
			kvmap,
		}
	}

	pub fn set(&self, key: String, value: String) {
		let mut encrypted = EncryptedMem::new();
		let _ = encrypted.encrypt(&value);
		self.kvmap.write().unwrap().insert(key, encrypted);
	}

	pub fn get(&self, key: &str) -> Option<String> {
		self.kvmap
			.read()
			.unwrap()
			.get(key)
			.and_then(|encrypted: &EncryptedMem| {
				encrypted
					.decrypt()
					.ok()
					.and_then(|bytes: ZeroizeBytes| String::from_utf8(bytes.as_ref().to_vec()).ok())
			})
	}

	pub fn remove(&self, key: &str) -> bool {
		self.kvmap.write().unwrap().remove(key).is_some()
	}

	pub fn contains_key(&self, key: &str) -> bool {
		self.kvmap.read().unwrap().contains_key(key)
	}

	pub fn len(&self) -> usize {
		self.kvmap.read().unwrap().len()
	}

	pub fn is_empty(&self) -> bool {
		self.kvmap.read().unwrap().is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_database_is_empty() {
		let db = Database::default();
		assert!(db.is_empty());
		assert_eq!(db.len(), 0);
	}

	#[test]
	fn test_set_and_get() {
		let db = Database::default();
		db.set("username".to_string(), "alice".to_string());
		assert_eq!(db.get("username"), Some("alice".to_string()));
		assert_eq!(db.len(), 1);
		assert!(!db.is_empty());
	}

	#[test]
	fn test_get_nonexistent_key() {
		let db = Database::default();
		assert_eq!(db.get("nonexistent"), None);
	}

	#[test]
	fn test_overwrite_value() {
		let db = Database::default();
		db.set("key".to_string(), "value1".to_string());
		db.set("key".to_string(), "value2".to_string());
		assert_eq!(db.get("key"), Some("value2".to_string()));
		assert_eq!(db.len(), 1);
	}

	#[test]
	fn test_multiple_keys() {
		let db = Database::default();
		db.set("key1".to_string(), "value1".to_string());
		db.set("key2".to_string(), "value2".to_string());
		db.set("key3".to_string(), "value3".to_string());
		assert_eq!(db.get("key1"), Some("value1".to_string()));
		assert_eq!(db.get("key2"), Some("value2".to_string()));
		assert_eq!(db.get("key3"), Some("value3".to_string()));
		assert_eq!(db.len(), 3);
	}

	#[test]
	fn test_remove_existing_key() {
		let db = Database::default();
		db.set("key".to_string(), "value".to_string());
		assert!(db.contains_key("key"));
		assert!(db.remove("key"));
		assert!(!db.contains_key("key"));
		assert_eq!(db.get("key"), None);
		assert_eq!(db.len(), 0);
		assert!(db.is_empty());
	}

	#[test]
	fn test_remove_nonexistent_key() {
		let db = Database::default();
		assert!(!db.remove("nonexistent"));
	}

	#[test]
	fn test_contains_key() {
		let db = Database::default();
		assert!(!db.contains_key("key"));
		db.set("key".to_string(), "value".to_string());
		assert!(db.contains_key("key"));
		db.remove("key");
		assert!(!db.contains_key("key"));
	}

	#[test]
	fn test_len_tracking() {
		let db = Database::default();
		assert_eq!(db.len(), 0);
		db.set("key1".to_string(), "value1".to_string());
		assert_eq!(db.len(), 1);
		db.set("key2".to_string(), "value2".to_string());
		assert_eq!(db.len(), 2);
		db.set("key1".to_string(), "updated".to_string());
		assert_eq!(db.len(), 2);
		db.remove("key1");
		assert_eq!(db.len(), 1);
		db.remove("key2");
		assert_eq!(db.len(), 0);
	}

	#[test]
	fn test_empty_string_value() {
		let db = Database::default();
		db.set("empty".to_string(), "".to_string());
		assert_eq!(db.get("empty"), Some("".to_string()));
		assert!(db.contains_key("empty"));
	}

	#[test]
	fn test_empty_string_key() {
		let db = Database::default();
		db.set("".to_string(), "value".to_string());
		assert_eq!(db.get(""), Some("value".to_string()));
		assert!(db.contains_key(""));
	}

	#[test]
	fn test_special_characters() {
		let db = Database::default();
		let special = "!@#$%^&*()_+-=[]{}|;:',.<>?/~`";
		db.set("special".to_string(), special.to_string());
		assert_eq!(db.get("special"), Some(special.to_string()));
	}

	#[test]
	fn test_unicode_values() {
		let db = Database::default();
		db.set("emoji".to_string(), "🔒🔑🛡️".to_string());
		db.set("chinese".to_string(), "你好世界".to_string());
		db.set("arabic".to_string(), "مرحبا".to_string());
		assert_eq!(db.get("emoji"), Some("🔒🔑🛡️".to_string()));
		assert_eq!(db.get("chinese"), Some("你好世界".to_string()));
		assert_eq!(db.get("arabic"), Some("مرحبا".to_string()));
	}

	#[test]
	fn test_long_values() {
		let db = Database::default();
		let long_value = "a".repeat(10000);
		db.set("long".to_string(), long_value.clone());
		assert_eq!(db.get("long"), Some(long_value));
	}

	#[test]
	fn test_sensitive_data_encryption() {
		let db = Database::default();
		let password = "super_secret_password_123!@#";
		let api_key = "sk_live_1234567890abcdefghijklmnop";
		db.set("password".to_string(), password.to_string());
		db.set("api_key".to_string(), api_key.to_string());
		assert_eq!(db.get("password"), Some(password.to_string()));
		assert_eq!(db.get("api_key"), Some(api_key.to_string()));
	}

	#[test]
	fn test_multiple_operations_sequence() {
		let db = Database::default();
		db.set("a".to_string(), "1".to_string());
		db.set("b".to_string(), "2".to_string());
		assert_eq!(db.len(), 2);
		db.remove("a");
		assert_eq!(db.len(), 1);
		assert_eq!(db.get("b"), Some("2".to_string()));
		db.set("c".to_string(), "3".to_string());
		assert_eq!(db.len(), 2);
		db.set("b".to_string(), "updated".to_string());
		assert_eq!(db.len(), 2);
		assert_eq!(db.get("b"), Some("updated".to_string()));
	}

	#[test]
	fn test_drop_cleanup() {
		let db = Database::default();
		db.set("key1".to_string(), "sensitive_data_1".to_string());
		db.set("key2".to_string(), "sensitive_data_2".to_string());
		drop(db);
	}

	#[test]
	fn test_case_sensitive_keys() {
		let db = Database::default();
		db.set("Key".to_string(), "value1".to_string());
		db.set("key".to_string(), "value2".to_string());
		db.set("KEY".to_string(), "value3".to_string());
		assert_eq!(db.len(), 3);
		assert_eq!(db.get("Key"), Some("value1".to_string()));
		assert_eq!(db.get("key"), Some("value2".to_string()));
		assert_eq!(db.get("KEY"), Some("value3".to_string()));
	}

	#[test]
	fn test_whitespace_in_values() {
		let db = Database::default();
		db.set("spaces".to_string(), "value with spaces".to_string());
		db.set("tabs".to_string(), "value\twith\ttabs".to_string());
		db.set("newlines".to_string(), "value\nwith\nnewlines".to_string());
		assert_eq!(db.get("spaces"), Some("value with spaces".to_string()));
		assert_eq!(db.get("tabs"), Some("value\twith\ttabs".to_string()));
		assert_eq!(
			db.get("newlines"),
			Some("value\nwith\nnewlines".to_string())
		);
	}
}
