use crate::storage::volatile;

use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	env, fs,
	io::Write,
	path::PathBuf,
	sync::{Arc, RwLock},
};
use zeroize::{Zeroize, Zeroizing};

const INNER_AVRO_SCHEMA: &str = r#"
{
	"type": "record",
	"name": "inner",
	"fields": [
		{"name": "db64", "type": "string"},
		{"name": "meta", "type": {"type": "map", "values": "string"}}
	]
}
"#;

const OUTER_AVRO_SCHEMA: &str = r#"
{
	"type": "record",
	"name": "outer",
	"fields": [
		{"name": "kvmap", "type": {"type": "map", "values": "string"}},
		{"name": "meta", "type": {"type": "map", "values": "string"}}
	]
}
"#;

#[derive(Deserialize, Serialize)]
pub struct InnerAvroDatabase {
	pub db64: String,
	pub meta: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
pub struct OuterAvroDatabase {
	pub kvmap: BTreeMap<String, String>,
	pub meta: BTreeMap<String, String>,
}

impl InnerAvroDatabase {
	fn into_avro(self) -> Vec<u8> {
		use apache_avro::{to_value, Schema, Writer};
		let schema = Schema::parse_str(INNER_AVRO_SCHEMA).unwrap();
		let mut writer = Writer::new(&schema, Vec::new());
		writer.append(to_value(self).unwrap()).unwrap();
		writer.into_inner().unwrap()
	}

	fn from_avro(avro_dat: Vec<u8>) -> Self {
		use apache_avro::{from_value, Reader};
		let reader = Reader::new(&avro_dat[..]).unwrap();
		from_value::<Self>(&reader.last().unwrap().unwrap()).unwrap()
	}

	fn into_vec(self) -> Vec<u8> {
		into_erasure_blocks(compress(self.into_avro()))
	}

	fn from_vec(dat: Vec<u8>) -> Self {
		Self::from_avro(decompress(from_erasure_blocks(dat)))
	}

	fn from_outer(outer_db: OuterAvroDatabase, master_key: [u8; 32]) -> Self {
		let meta = outer_db.meta.clone();
		let db64 = to_base64(&encrypt(
			outer_db.into_avro(),
			master_key.to_vec(),
			[0u8; 24].to_vec(),
		));
		Self { db64, meta }
	}

	fn into_outer(self, master_key: [u8; 32]) -> OuterAvroDatabase {
		OuterAvroDatabase::from_avro(
			decrypt(
				from_base64(&self.db64),
				master_key.to_vec(),
				[0u8; 24].to_vec(),
			)
			.unwrap(),
		)
	}
}

impl OuterAvroDatabase {
	fn into_avro(self) -> Vec<u8> {
		use apache_avro::{to_value, Schema, Writer};
		let schema = Schema::parse_str(OUTER_AVRO_SCHEMA).unwrap();
		let mut writer = Writer::new(&schema, Vec::new());
		writer.append(to_value(self).unwrap()).unwrap();
		writer.into_inner().unwrap()
	}

	fn from_avro(avro_dat: Vec<u8>) -> Self {
		use apache_avro::{from_value, Reader};
		let reader = Reader::new(&avro_dat[..]).unwrap();
		from_value::<Self>(&reader.last().unwrap().unwrap()).unwrap()
	}

	fn into_inner(self, master_key: [u8; 32]) -> InnerAvroDatabase {
		InnerAvroDatabase::from_outer(self, master_key)
	}

	fn from_inner(inner_db: InnerAvroDatabase, master_key: [u8; 32]) -> Self {
		inner_db.into_outer(master_key)
	}

	fn into_volatile(self, master_key: [u8; 32]) -> volatile::Database {
		let meta = Arc::new(RwLock::new(self.meta.clone()));
		let kvmap = Arc::new(RwLock::new(self.kvmap.clone()));
		volatile::Database::old(master_key, meta, kvmap)
	}

	fn from_volatile(volatile_db: &volatile::Database) -> Self {
		let meta = volatile_db.meta.read().unwrap().clone();
		let mut kvmap = BTreeMap::new();
		for (key, encrypted_value) in volatile_db.kvmap.read().unwrap().iter() {
			let value =
				String::from_utf8(encrypted_value.decrypt().unwrap().as_ref().to_vec()).unwrap();
			kvmap.insert(key.to_string(), value);
		}
		Self { kvmap, meta }
	}
}

pub fn db_path(db_name: &str) -> PathBuf {
	let mut apath = env::home_dir().unwrap_or_default();
	apath.push(format!(".config/digisafe/{}.digisafe", db_name));
	fs::create_dir_all(apath.parent().unwrap()).ok();
	apath
}

pub fn load(db_name: String, master_password: String) -> volatile::Database {
	let master_key = master_key_derivation(master_password);
	let path = db_path(&db_name);
	if path.exists() {
		let dat = fs::read(path).unwrap();
		let inner_db = InnerAvroDatabase::from_vec(dat);
		let outer_db = OuterAvroDatabase::from_inner(inner_db, master_key);
		outer_db.into_volatile(master_key)
	} else {
		volatile::Database::new(master_key, db_name)
	}
}

pub fn save(db: volatile::Database) -> String {
	// Todo: atomic swap temp file.
	let master_key = db.master_key.read().unwrap().decrypt().unwrap();
	let outer_db = OuterAvroDatabase::from_volatile(&db);
	let inner_db = outer_db.into_inner(master_key.as_ref().try_into().unwrap());
	let db_name = db.meta.read().unwrap().get("db_name").unwrap().clone();
	let path = db_path(&db_name);
	let path_str = path.to_str().unwrap().to_string();
	let dat = inner_db.into_vec();
	fs::write(path, dat).unwrap();
	format!("Database saved. {}", path_str)
}

pub fn master_key_derivation(password: String) -> [u8; 32] {
	use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
	use sha3::{Digest, Sha3_256};
	let password = Zeroizing::new(password);
	let salt = Zeroizing::new("digisafe".to_string());
	let mut pre_hasher = Sha3_256::new();
	pre_hasher.update(salt.as_bytes());
	pre_hasher.update(password.as_bytes());
	let mut pre_hash = pre_hasher.finalize();
	let main_params = ParamsBuilder::new()
		.m_cost(2u32.pow(20))
		.t_cost(1)
		.p_cost(4)
		.output_len(32)
		.build()
		.unwrap();
	let main_hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, main_params);
	let mut main_hash = [0u8; 32];
	main_hasher
		.hash_password_into(&pre_hash, salt.as_bytes(), &mut main_hash)
		.unwrap();
	let mut post_hasher = Sha3_256::new();
	post_hasher.update(main_hash);
	post_hasher.update(salt.as_bytes());
	let post_hash: [u8; 32] = post_hasher.finalize().into();
	pre_hash.zeroize();
	main_hash.zeroize();
	post_hash
}

pub fn to_base64(msg: &[u8]) -> String {
	use base64ct::{Base64, Encoding};
	Base64::encode_string(msg)
}

pub fn from_base64(msg_enc: &str) -> Vec<u8> {
	use base64ct::{Base64, Encoding};
	Base64::decode_vec(msg_enc).unwrap()
}

pub fn compress(msg: Vec<u8>) -> Vec<u8> {
	use lz4::EncoderBuilder;
	let mut encoder = EncoderBuilder::new().level(9).build(vec![]).unwrap();
	let _ = encoder.write(&msg[..]);
	encoder.finish().0
}

pub fn decompress(msg_enc: Vec<u8>) -> Vec<u8> {
	use lz4::Decoder;
	let mut msg = vec![];
	{
		let mut decoder = Decoder::new(&msg_enc[..]).unwrap();
		let _ = std::io::copy(&mut decoder, &mut msg);
	}
	msg
}

pub fn encrypt(msg: Vec<u8>, key: Vec<u8>, nonce: Vec<u8>) -> Vec<u8> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let key: [u8; 32] = key.try_into().unwrap();
	let nonce: [u8; 24] = nonce.try_into().unwrap();
	let cipher = XChaCha20Poly1305::new(&key.into());
	cipher.encrypt(&nonce.into(), &msg[..]).unwrap()
}

pub fn decrypt(msg_enc: Vec<u8>, key: Vec<u8>, nonce: Vec<u8>) -> Option<Vec<u8>> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let key: [u8; 32] = key.try_into().unwrap();
	let nonce: [u8; 24] = nonce.try_into().unwrap();
	let cipher = XChaCha20Poly1305::new(&key.into());
	cipher.decrypt(&nonce.into(), msg_enc.as_ref()).ok()
}

fn to_erasure_block(dat: &[u8]) -> Vec<u8> {
	use reed_solomon::Encoder;
	let enc = Encoder::new(8);
	Vec::from(&enc.encode(dat)[..])
}

fn from_erasure_block(dat_enc: &[u8]) -> Vec<u8> {
	use reed_solomon::Decoder;
	let dec = Decoder::new(8);
	dec.correct(dat_enc, None).unwrap().data().to_owned()
}

pub fn into_erasure_blocks(dat: Vec<u8>) -> Vec<u8> {
	let block_size = 247usize;
	let num_blocks = dat.chunks(block_size).count();
	let mut dat_enc = Vec::with_capacity(dat.len() + 8 * num_blocks);
	for chunk in dat.chunks(block_size) {
		let enc_dat_slice = to_erasure_block(chunk);
		dat_enc.extend_from_slice(&enc_dat_slice);
	}
	dat_enc
}

pub fn from_erasure_blocks(dat_enc: Vec<u8>) -> Vec<u8> {
	let block_size = 255usize;
	let num_blocks = dat_enc.chunks(block_size).count();
	let mut dat = Vec::with_capacity(dat_enc.len() - 8 * num_blocks);
	for chunk in dat_enc.chunks(block_size) {
		let dat_slice = from_erasure_block(chunk);
		dat.extend_from_slice(&dat_slice);
	}
	dat
}
