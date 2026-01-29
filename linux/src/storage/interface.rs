/*
 * The middle ground between gui::AppState and volatile::Database.
 */

use crate::storage::volatile;
use memsecurity::EncryptedMem;
use std::{
	collections::{BTreeMap, HashMap},
	iter, vec,
};

trait NullSeparatedKeys {
	fn get_nsk_tuples(&self, primary_key: &str, secondary_key: &str) -> Vec<(String, String)>;
	fn put_nsk_tuples(&mut self, primary_key: &str, tuples: Vec<(String, String)>);
}

impl NullSeparatedKeys for BTreeMap<String, EncryptedMem> {
	fn get_nsk_tuples(&self, primary_key: &str, secondary_key: &str) -> Vec<(String, String)> {
		let range_start = format!("{}\x00{}\x00", primary_key, secondary_key);
		let range_end = format!("{}\x00{}\x01", primary_key, secondary_key);
		self.range(range_start..range_end)
			.map(|(k, v)| {
				(
					k.split("\x00").nth(2).unwrap_or("").to_string(),
					v.decrypt().unwrap().to_string(),
				)
			})
			.chain(iter::once((
				"secondary_key".to_string(),
				secondary_key.to_string(),
			)))
			.collect::<Vec<(String, String)>>()
	}

	fn put_nsk_tuples(&mut self, primary_key: &str, tuples: Vec<(String, String)>) {
		let secondary_key = &tuples
			.iter()
			.find(|(k, _)| k == "secondary_key")
			.unwrap()
			.1
			.clone();
		for tuple in tuples {
			if tuple.1 != "secondary_key" {
				self.insert(
					format!("{}\x00{}\x00{}", primary_key, secondary_key, tuple.0),
					tuple.1,
				);
			}
		}
	}
}

#[derive(Default)]
pub struct PasswordEntry {
	title: String,
	username: String,
	password: EncryptedMem,
	note: String,
	tags: String,
	url: String,
	expires_ts: String,
}

impl FromIterator<(String, String)> for PasswordEntry {
	fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
		let mut map: HashMap<_, _> = iter.into_iter().collect();
		Self {
			title: map.remove("secondary_key").unwrap_or_default(),
			username: map.remove("username").unwrap_or_default(),
			password: map.remove("password").unwrap_or_default(),
			note: map.remove("note").unwrap_or_default(),
			tags: map.remove("tags").unwrap_or_default(),
			url: map.remove("url").unwrap_or_default(),
			expires_ts: map.remove("expires_ts").unwrap_or_default(),
		}
	}
}

impl IntoIterator for PasswordEntry {
	type Item = (String, String);
	type IntoIter = vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		vec![
			("secondary_key".into(), self.title),
			("username".into(), self.username),
			("password".into(), self.password),
			("note".into(), self.note),
			("tags".into(), self.tags),
			("url".into(), self.url),
			("expires_ts".into(), self.expires_ts),
		]
		.into_iter()
	}
}

pub struct Database {
	vdb: volatile::Database,
}

impl Database {
	pub fn get_password_entry(&self, secondary_key: &str) -> PasswordEntry {
		self.vdb
			.private_kv
			.read()
			.unwrap()
			.get_nsk_tuples("password_entry", secondary_key)
			.into_iter()
			.collect::<PasswordEntry>()
	}
}
