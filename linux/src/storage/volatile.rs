#![allow(dead_code)]
use memsecurity::{EncryptedMem, ZeroizeBytes};
use std::{
	collections::BTreeMap,
	mem,
	sync::{Arc, RwLock},
	time::SystemTime,
};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Debug, Default)]
pub struct Database {
	pub master_key: Arc<RwLock<EncryptedMem>>,
	pub private_kv: Arc<RwLock<BTreeMap<String, EncryptedMem>>>,
	pub public_kv: Arc<RwLock<BTreeMap<String, String>>>,
}

impl Drop for Database {
	fn drop(&mut self) {
		if let Some(rwlock) = Arc::get_mut(&mut self.private_kv) && let Ok(private_kv) = rwlock.get_mut() {
			for (mut key, _value) in mem::take(private_kv).into_iter() {
				key.zeroize();
			}
		}
	}
}

impl Database {
	pub fn old(
		master_key: [u8; 32],
		private_kv: Arc<RwLock<BTreeMap<String, String>>>,
		public_kv: Arc<RwLock<BTreeMap<String, String>>>,
	) -> Database {
		let master_key = Zeroizing::new(master_key);
		let encrypted_master_key = Arc::new(RwLock::new(EncryptedMem::new()));
		let _ = encrypted_master_key
			.write()
			.unwrap()
			.encrypt(&master_key)
			.unwrap();
		let private_kv_encrypted = Arc::new(RwLock::new(BTreeMap::<String, EncryptedMem>::new()));
		for (key, value) in private_kv.write().unwrap().iter_mut() {
			let mut encrypted_value = EncryptedMem::new();
			let _ = encrypted_value.encrypt(value);
			private_kv_encrypted
				.write()
				.unwrap()
				.insert(key.to_string(), encrypted_value);
			value.zeroize();
		}
		Database {
			master_key: encrypted_master_key,
			private_kv: private_kv_encrypted,
			public_kv,
		}
	}

	pub fn new(master_key: [u8; 32], digisalt: [u8; 32], db_name: String) -> Database {
		let master_key = Zeroizing::new(master_key);
		let encrypted_master_key = Arc::new(RwLock::new(EncryptedMem::new()));
		let _ = encrypted_master_key
			.write()
			.unwrap()
			.encrypt(&master_key)
			.unwrap();
		let mut public_kv = BTreeMap::new();
		public_kv.insert("db_name".to_string(), db_name.to_string());
		public_kv.insert("digisalt".to_string(), hex::encode(digisalt));
		let created_ts = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.to_string();
		public_kv.insert("created_ts".to_string(), created_ts.clone());
		public_kv.insert("modified_ts".to_string(), created_ts);
		public_kv.insert("nonce".to_string(), "0".to_string());
		let private_kv = Arc::new(RwLock::new(BTreeMap::new()));
		Database {
			master_key: encrypted_master_key,
			private_kv,
			public_kv: Arc::new(RwLock::new(public_kv)),
		}
	}

	pub fn set_private(&self, key: String, value: String) {
		let mut encrypted = EncryptedMem::new();
		let _ = encrypted.encrypt(&value);
		self.private_kv.write().unwrap().insert(key, encrypted);
		let mut value = value;
		value.zeroize();
	}

	pub fn get_private(&self, key: &str) -> Option<String> {
		self.private_kv
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

	pub fn remove_private(&self, key: &str) -> bool {
		self.private_kv.write().unwrap().remove(key).is_some()
	}

	pub fn contains_key_private(&self, key: &str) -> bool {
		self.private_kv.read().unwrap().contains_key(key)
	}

	pub fn len_private(&self) -> usize {
		self.private_kv.read().unwrap().len()
	}

	pub fn is_empty_private(&self) -> bool {
		self.private_kv.read().unwrap().is_empty()
	}

	pub fn set_public(&self, key: String, value: String) {
		self.public_kv.write().unwrap().insert(key, value);
	}

	pub fn get_public(&self, key: &str) -> Option<String> {
		self.public_kv.read().unwrap().get(key).cloned()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_database_is_empty() {
		let db = Database::default();
		assert!(db.is_empty_private());
		assert_eq!(db.len_private(), 0);
	}

	#[test]
	fn test_set_and_get() {
		let db = Database::default();
		db.set_private("username".to_string(), "alice".to_string());
		assert_eq!(db.get_private("username"), Some("alice".to_string()));
		assert_eq!(db.len_private(), 1);
		assert!(!db.is_empty_private());
	}

	#[test]
	fn test_get_nonexistent_key() {
		let db = Database::default();
		assert_eq!(db.get_private("nonexistent"), None);
	}

	#[test]
	fn test_overwrite_value() {
		let db = Database::default();
		db.set_private("key".to_string(), "value1".to_string());
		db.set_private("key".to_string(), "value2".to_string());
		assert_eq!(db.get_private("key"), Some("value2".to_string()));
		assert_eq!(db.len_private(), 1);
	}

	#[test]
	fn test_multiple_keys() {
		let db = Database::default();
		db.set_private("key1".to_string(), "value1".to_string());
		db.set_private("key2".to_string(), "value2".to_string());
		db.set_private("key3".to_string(), "value3".to_string());
		assert_eq!(db.get_private("key1"), Some("value1".to_string()));
		assert_eq!(db.get_private("key2"), Some("value2".to_string()));
		assert_eq!(db.get_private("key3"), Some("value3".to_string()));
		assert_eq!(db.len_private(), 3);
	}

	#[test]
	fn test_remove_existing_key() {
		let db = Database::default();
		db.set_private("key".to_string(), "value".to_string());
		assert!(db.contains_key_private("key"));
		assert!(db.remove_private("key"));
		assert!(!db.contains_key_private("key"));
		assert_eq!(db.get_private("key"), None);
		assert_eq!(db.len_private(), 0);
		assert!(db.is_empty_private());
	}

	#[test]
	fn test_remove_nonexistent_key() {
		let db = Database::default();
		assert!(!db.remove_private("nonexistent"));
	}

	#[test]
	fn test_contains_key() {
		let db = Database::default();
		assert!(!db.contains_key_private("key"));
		db.set_private("key".to_string(), "value".to_string());
		assert!(db.contains_key_private("key"));
		db.remove_private("key");
		assert!(!db.contains_key_private("key"));
	}

	#[test]
	fn test_len_tracking() {
		let db = Database::default();
		assert_eq!(db.len_private(), 0);
		db.set_private("key1".to_string(), "value1".to_string());
		assert_eq!(db.len_private(), 1);
		db.set_private("key2".to_string(), "value2".to_string());
		assert_eq!(db.len_private(), 2);
		db.set_private("key1".to_string(), "updated".to_string());
		assert_eq!(db.len_private(), 2);
		db.remove_private("key1");
		assert_eq!(db.len_private(), 1);
		db.remove_private("key2");
		assert_eq!(db.len_private(), 0);
	}

	#[test]
	fn test_empty_string_value() {
		let db = Database::default();
		db.set_private("empty".to_string(), "".to_string());
		assert_eq!(db.get_private("empty"), Some("".to_string()));
		assert!(db.contains_key_private("empty"));
	}

	#[test]
	fn test_empty_string_key() {
		let db = Database::default();
		db.set_private("".to_string(), "value".to_string());
		assert_eq!(db.get_private(""), Some("value".to_string()));
		assert!(db.contains_key_private(""));
	}

	#[test]
	fn test_special_characters() {
		let db = Database::default();
		let special = "!@#$%^&*()_+-=[]{}|;:',.<>?/~`";
		db.set_private("special".to_string(), special.to_string());
		assert_eq!(db.get_private("special"), Some(special.to_string()));
	}

	#[test]
	fn test_unicode_values() {
		let db = Database::default();
		db.set_private("emoji".to_string(), "🔒🔑🛡️".to_string());
		db.set_private("chinese".to_string(), "你好世界".to_string());
		db.set_private("arabic".to_string(), "مرحبا".to_string());
		assert_eq!(db.get_private("emoji"), Some("🔒🔑🛡️".to_string()));
		assert_eq!(db.get_private("chinese"), Some("你好世界".to_string()));
		assert_eq!(db.get_private("arabic"), Some("مرحبا".to_string()));
	}

	#[test]
	fn test_long_values() {
		let db = Database::default();
		let long_value = "a".repeat(10000);
		db.set_private("long".to_string(), long_value.clone());
		assert_eq!(db.get_private("long"), Some(long_value));
	}

	#[test]
	fn test_sensitive_data_encryption() {
		let db = Database::default();
		let password = "super_secret_password_123!@#";
		let api_key = "sk_live_1234567890abcdefghijklmnop";
		db.set_private("password".to_string(), password.to_string());
		db.set_private("api_key".to_string(), api_key.to_string());
		assert_eq!(db.get_private("password"), Some(password.to_string()));
		assert_eq!(db.get_private("api_key"), Some(api_key.to_string()));
	}

	#[test]
	fn test_multiple_operations_sequence() {
		let db = Database::default();
		db.set_private("a".to_string(), "1".to_string());
		db.set_private("b".to_string(), "2".to_string());
		assert_eq!(db.len_private(), 2);
		db.remove_private("a");
		assert_eq!(db.len_private(), 1);
		assert_eq!(db.get_private("b"), Some("2".to_string()));
		db.set_private("c".to_string(), "3".to_string());
		assert_eq!(db.len_private(), 2);
		db.set_private("b".to_string(), "updated".to_string());
		assert_eq!(db.len_private(), 2);
		assert_eq!(db.get_private("b"), Some("updated".to_string()));
	}

	#[test]
	fn test_drop_cleanup() {
		let db = Database::default();
		db.set_private("key1".to_string(), "sensitive_data_1".to_string());
		db.set_private("key2".to_string(), "sensitive_data_2".to_string());
		drop(db);
	}

	#[test]
	fn test_case_sensitive_keys() {
		let db = Database::default();
		db.set_private("Key".to_string(), "value1".to_string());
		db.set_private("key".to_string(), "value2".to_string());
		db.set_private("KEY".to_string(), "value3".to_string());
		assert_eq!(db.len_private(), 3);
		assert_eq!(db.get_private("Key"), Some("value1".to_string()));
		assert_eq!(db.get_private("key"), Some("value2".to_string()));
		assert_eq!(db.get_private("KEY"), Some("value3".to_string()));
	}

	#[test]
	fn test_whitespace_in_values() {
		let db = Database::default();
		db.set_private("spaces".to_string(), "value with spaces".to_string());
		db.set_private("tabs".to_string(), "value\twith\ttabs".to_string());
		db.set_private("newlines".to_string(), "value\nwith\nnewlines".to_string());
		assert_eq!(db.get_private("spaces"), Some("value with spaces".to_string()));
		assert_eq!(db.get_private("tabs"), Some("value\twith\ttabs".to_string()));
		assert_eq!(
			db.get_private("newlines"),
			Some("value\nwith\nnewlines".to_string())
		);
	}
}
