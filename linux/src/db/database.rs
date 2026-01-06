// State<Unlocked> <-> Avro <-> LZ4 <-> ChaCha20Poly1305 <-> Base64 <-> State<Locked>
// State<Locked> -> Avro -> Reed-Solomon -> Vec<u8> -> name.digisafe

use crate::db::crypto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

const UNLOCKED_SCHEMA_RAW: &str = r#"
{
	"type": "record",
	"name": "unlocked",
	"fields": [
		{"name": "db", "type": {"type": "map", "values": {"type": "map", "values": "string"}}},
		{"name": "meta", "type": {"type": "map", "values": "string"}}
	]
}
"#;

const LOCKED_SCHEMA_RAW: &str = r#"
{
	"type": "record",
	"name": "locked",
	"fields": [
		{"name": "db", "type": "string"},
		{"name": "meta", "type": {"type": "map", "values": "string"}}
	]
}
"#;

#[derive(Debug, Deserialize, Serialize)]
pub struct AvroUnlocked {
	db: HashMap<String, HashMap<String, String>>,
	meta: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvroLocked {
	db: String,
	meta: HashMap<String, String>,
}

#[derive(Default, Debug, Clone)]
pub struct Unlocked {
	db: HashMap<String, HashMap<String, String>>,
	meta: HashMap<String, String>,
	key: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct Locked {
	db: String,
	meta: HashMap<String, String>,
	key: Vec<u8>,
}

pub enum State {
	UnlockedWrapper(Unlocked),
	LockedWrapper(Locked),
}

impl AvroUnlocked {
	pub fn to_unlocked_struct(self, key: Vec<u8>) -> Unlocked {
		Unlocked {
			db: self.db,
			meta: self.meta,
			key,
		}
	}
}

impl AvroLocked {
	pub fn to_locked_struct(self) -> Locked {
		Locked {
			db: self.db,
			meta: self.meta,
			key: Default::default(),
		}
	}
}

impl Unlocked {
	pub fn new() -> Self {
		let mut db = Unlocked {
			db: Default::default(),
			meta: Default::default(),
			key: Default::default(),
		};
		let ts = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.to_string();
		let mut randy = [0u8; 64];
		getrandom::fill(&mut randy).expect("Failed to get entropy.");
		let randy_b64 = crypto::to_base64(randy.as_ref());
		db.set_meta("uid", randy_b64);
		db.set_meta("app", "digisafe");
		db.set_meta("revision", "0");
		db.set_meta("version", "1.0.0");
		db.set_meta("timestamp", ts);
		db
	}

	pub fn lock(self) -> Locked {
		let cadb = crypto::compress(self.to_avro());
		let nonce = crypto::hash_lite(&cadb)[..24].to_vec();
		let mut meta = HashMap::<String, String>::new();
		meta.extend(self.meta);
		meta.insert("nonce".into(), crypto::to_base64(&nonce));
		let db = crypto::to_base64(&crypto::encrypt(cadb, self.key.clone(), nonce));
		Locked {
			db,
			meta,
			key: self.key,
		}
	}

	pub fn get(&self, akey: &String) -> Option<String> {
		self.db.get(akey).and_then(|hm| hm.get("value")).cloned()
	}

	pub fn set(&mut self, akey: impl Into<String>, aval: impl Into<String>) {
		self.db.entry(akey.into()).and_modify(|hm| {
			hm.insert("value".into(), aval.into());
		});
	}

	pub fn get_meta(&self, akey: &String) -> Option<String> {
		self.meta.get(akey).cloned()
	}

	pub fn set_meta(&mut self, akey: impl Into<String>, aval: impl Into<String>) {
		self.meta.insert(akey.into(), aval.into());
	}

	pub fn merge(&mut self, other: Unlocked) {
		self.db.extend(other.db);
	}

	pub fn to_avro_struct(&self) -> AvroUnlocked {
		AvroUnlocked {
			db: self.db.clone(),
			meta: self.meta.clone(),
		}
	}

	pub fn to_avro(&self) -> Vec<u8> {
		use apache_avro::{to_value, Schema, Writer};
		let schema = Schema::parse_str(UNLOCKED_SCHEMA_RAW).unwrap();
		let mut writer = Writer::new(&schema, Vec::new());
		writer
			.append(to_value(self.to_avro_struct()).unwrap())
			.unwrap();
		writer.into_inner().unwrap()
	}

	pub fn from_avro(avro_dat: Vec<u8>, key: Vec<u8>) -> Unlocked {
		use apache_avro::{from_value, Reader};
		let reader = Reader::new(&avro_dat[..]).unwrap();
		let db = from_value::<AvroUnlocked>(&reader.last().unwrap().unwrap()).unwrap();
		db.to_unlocked_struct(key)
	}
}

impl Locked {
	pub fn unlock(self, password: impl Into<String>) -> Option<Unlocked> {
		let key = crypto::hash_heavy(password.into());
		let nonce = crypto::from_base64(self.meta.get("nonce").unwrap());
		let cradb = crypto::decrypt(crypto::from_base64(&self.db), key.clone(), nonce);
		cradb.as_ref()?;
		let radb = crypto::decompress(cradb.unwrap());
		let db = Unlocked::from_avro(radb, key);
		Some(db)
	}

	pub fn to_avro_struct(&self) -> AvroLocked {
		AvroLocked {
			db: self.db.clone(),
			meta: self.meta.clone(),
		}
	}

	pub fn to_avro(&self) -> Vec<u8> {
		use apache_avro::{to_value, Schema, Writer};
		let schema = Schema::parse_str(LOCKED_SCHEMA_RAW).unwrap();
		let mut writer = Writer::new(&schema, Vec::new());
		writer
			.append(to_value(self.to_avro_struct()).unwrap())
			.unwrap();
		writer.into_inner().unwrap()
	}

	pub fn from_avro(avro_dat: Vec<u8>) -> Locked {
		use apache_avro::{from_value, Reader};
		let reader = Reader::new(&avro_dat[..]).unwrap();
		let db = from_value::<AvroLocked>(&reader.last().unwrap().unwrap()).unwrap();
		db.to_locked_struct()
	}

	pub fn to_vec(&self) -> Vec<u8> {
		crypto::to_ecc(self.to_avro())
	}

	pub fn from_vec(dat_ecc: Vec<u8>) -> Locked {
		Locked::from_avro(crypto::from_ecc(dat_ecc))
	}

	pub fn db_path(&self) -> PathBuf {
		let mut apath = env::home_dir().unwrap_or_default();
		apath.push(format!(
			".config/digisafe/digisafe_{}.db",
			self.meta.get("id").unwrap()
		));
		fs::create_dir_all(apath.parent().unwrap()).ok();
		apath
	}

	pub fn save(&self) {
		let mut fout = File::create(self.db_path()).unwrap();
		fout.write_all(&self.to_vec()[..]).unwrap();
	}
}
