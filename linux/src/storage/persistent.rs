use crate::storage::volatile;

use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	env, fs,
	io::Write,
	mem,
	path::PathBuf,
	process::Command,
	sync::{Arc, RwLock},
	time::SystemTime,
};
use zeroize::{Zeroize, Zeroizing};

const INNER_AVRO_SCHEMA: &str = r#"
{
	"type": "record",
	"name": "inner",
	"fields": [
		{"name": "db64", "type": "string"},
		{"name": "public_kv", "type": {"type": "map", "values": "string"}}
	]
}
"#;

const OUTER_AVRO_SCHEMA: &str = r#"
{
	"type": "record",
	"name": "outer",
	"fields": [
		{"name": "private_kv", "type": {"type": "map", "values": "string"}},
		{"name": "public_kv", "type": {"type": "map", "values": "string"}}
	]
}
"#;

#[derive(Deserialize, Serialize)]
pub struct InnerAvroDatabase {
	db64: String,
	public_kv: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
pub struct OuterAvroDatabase {
	private_kv: BTreeMap<String, String>,
	public_kv: BTreeMap<String, String>,
}

impl InnerAvroDatabase {
	fn get_nonce(&self) -> [u8; 24] {
		parse_nonce_from_kv(&self.public_kv)
	}

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
		let nonce = outer_db.get_nonce();
		let public_kv = outer_db.public_kv.clone();
		let db64 = to_base64(&encrypt(outer_db.into_avro(), master_key, nonce));
		Self { db64, public_kv }
	}

	fn into_outer(self, master_key: [u8; 32]) -> OuterAvroDatabase {
		let nonce = self.get_nonce();
		OuterAvroDatabase::from_avro(decrypt(from_base64(&self.db64), master_key, nonce).unwrap())
	}
}

impl OuterAvroDatabase {
	fn get_nonce(&self) -> [u8; 24] {
		parse_nonce_from_kv(&self.public_kv)
	}

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
		let private_kv = Arc::new(RwLock::new(self.private_kv.clone()));
		let public_kv = Arc::new(RwLock::new(self.public_kv.clone()));
		volatile::Database::old(master_key, private_kv, public_kv)
	}

	fn from_volatile(volatile_db: &volatile::Database) -> Self {
		let public_kv = volatile_db.public_kv.read().unwrap().clone();
		let mut private_kv = BTreeMap::new();
		for (key, encrypted_value) in volatile_db.private_kv.read().unwrap().iter() {
			let value =
				String::from_utf8(encrypted_value.decrypt().unwrap().as_ref().to_vec()).unwrap();
			private_kv.insert(key.to_string(), value);
		}
		Self {
			private_kv,
			public_kv,
		}
	}
}

impl Drop for OuterAvroDatabase {
	fn drop(&mut self) {
		for (mut key, mut value) in mem::take(&mut self.private_kv).into_iter() {
			value.zeroize();
			key.zeroize();
		}
	}
}

fn parse_nonce_from_kv(kv: &BTreeMap<String, String>) -> [u8; 24] {
	let pre_nonce = kv.get("nonce").unwrap().parse::<u128>().unwrap();
	let mut nonce = [0u8; 24];
	nonce[..16].copy_from_slice(&pre_nonce.to_le_bytes());
	nonce
}

fn base_path() -> PathBuf {
	let mut apath = env::home_dir().unwrap_or_default();
	apath.push(".config/digisafe/");
	fs::create_dir_all(&apath).ok();
	apath
}

fn db_path(db_name: &str) -> PathBuf {
	let mut apath = base_path();
	apath.push(format!("{}.digisafe", db_name));
	apath
}

fn temp_path(db_name: &str) -> PathBuf {
	let mut apath = base_path();
	apath.push(format!(".{}.digisafe", db_name));
	apath
}

fn pepper_path() -> String {
	let mut apath = base_path();
	apath.push("digipepper.cred");
	apath.to_str().unwrap().to_string()
}

pub fn load(db_name: String, master_password: String) -> volatile::Database {
	let path = db_path(&db_name);
	if path.exists() {
		let dat = fs::read(path).unwrap();
		let inner_db = InnerAvroDatabase::from_vec(dat);
		let digisalt: [u8; 32] = hex::decode(inner_db.public_kv.get("digisalt").unwrap())
			.unwrap()
			.try_into()
			.unwrap();
		let master_key = master_key_derivation(master_password, digisalt);
		let outer_db = OuterAvroDatabase::from_inner(inner_db, master_key);
		outer_db.into_volatile(master_key)
	} else {
		let mut digisalt = [0u8; 32];
		getrandom::fill(&mut digisalt).unwrap();
		let master_key = master_key_derivation(master_password, digisalt);
		volatile::Database::new(master_key, digisalt, db_name)
	}
}

pub fn save(db: volatile::Database) -> String {
	let modified_ts = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.to_string();
	db.set_public("modified_ts".to_string(), modified_ts);
	let nonce = db.get_public("nonce").unwrap().parse::<u128>().unwrap() + 1;
	db.set_public("nonce".to_string(), nonce.to_string());
	let master_key = db.master_key.read().unwrap().decrypt().unwrap();
	let outer_db = OuterAvroDatabase::from_volatile(&db);
	let inner_db = outer_db.into_inner(master_key.as_ref().try_into().unwrap());
	let db_name = db.public_kv.read().unwrap().get("db_name").unwrap().clone();
	let tmp_path = temp_path(&db_name);
	let path = db_path(&db_name);
	let mut dat = inner_db.into_vec();
	fs::write(&tmp_path, &dat).unwrap();
	fs::rename(tmp_path, path).unwrap();
	dat.zeroize();
	"Database saved.".to_string()
}

fn load_pepper() -> [u8; 32] {
	let pepper_output = Command::new("systemd-creds")
		.args(["decrypt", "--user", "--name=digipepper", &pepper_path()])
		.output()
		.unwrap();
	let pepper_hex = Zeroizing::new(pepper_output.stdout);
	let mut pepper = [0u8; 32];
	hex::decode_to_slice(
		std::str::from_utf8(&pepper_hex).unwrap().trim(),
		&mut pepper,
	)
	.unwrap();
	pepper
}

fn master_key_derivation(password: String, salt: [u8; 32]) -> [u8; 32] {
	use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
	use sha3::{Digest, Sha3_256};
	let password = Zeroizing::new(password);
	let pepper = load_pepper();
	let mut pre_hasher = Sha3_256::new();
	pre_hasher.update(salt);
	pre_hasher.update(pepper);
	pre_hasher.update(password.as_bytes());
	let mut pre_hash = pre_hasher.finalize();
	let main_params = ParamsBuilder::new()
		.m_cost(2u32.pow(22))
		.t_cost(1)
		.p_cost(1)
		.output_len(32)
		.build()
		.unwrap();
	let main_hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, main_params);
	let mut main_hash = [0u8; 32];
	main_hasher
		.hash_password_into(
			&pre_hash,
			&[&salt as &[u8], &pepper as &[u8]].concat(),
			&mut main_hash,
		)
		.unwrap();
	let mut post_hasher = Sha3_256::new();
	post_hasher.update(main_hash);
	post_hasher.update(pepper);
	post_hasher.update(salt);
	let post_hash: [u8; 32] = post_hasher.finalize().into();
	let mut pepper = pepper;
	pepper.zeroize();
	pre_hash.zeroize();
	main_hash.zeroize();
	post_hash
}

fn to_base64(msg: &[u8]) -> String {
	use base64ct::{Base64, Encoding};
	Base64::encode_string(msg)
}

fn from_base64(msg_enc: &str) -> Vec<u8> {
	use base64ct::{Base64, Encoding};
	Base64::decode_vec(msg_enc).unwrap()
}

fn compress(msg: Vec<u8>) -> Vec<u8> {
	use lz4::EncoderBuilder;
	let mut encoder = EncoderBuilder::new().level(9).build(vec![]).unwrap();
	let _ = encoder.write(&msg[..]);
	encoder.finish().0
}

fn decompress(msg_enc: Vec<u8>) -> Vec<u8> {
	use lz4::Decoder;
	let mut msg = vec![];
	{
		let mut decoder = Decoder::new(&msg_enc[..]).unwrap();
		let _ = std::io::copy(&mut decoder, &mut msg);
	}
	msg
}

fn encrypt(msg: Vec<u8>, key: [u8; 32], nonce: [u8; 24]) -> Vec<u8> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let cipher = XChaCha20Poly1305::new(&key.into());
	cipher.encrypt(&nonce.into(), &msg[..]).unwrap()
}

fn decrypt(msg_enc: Vec<u8>, key: [u8; 32], nonce: [u8; 24]) -> Option<Vec<u8>> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
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

fn into_erasure_blocks(dat: Vec<u8>) -> Vec<u8> {
	let block_size = 247usize;
	let num_blocks = dat.chunks(block_size).count();
	let mut dat_enc = Vec::with_capacity(dat.len() + 8 * num_blocks);
	for chunk in dat.chunks(block_size) {
		let enc_dat_slice = to_erasure_block(chunk);
		dat_enc.extend_from_slice(&enc_dat_slice);
	}
	dat_enc
}

fn from_erasure_blocks(dat_enc: Vec<u8>) -> Vec<u8> {
	let block_size = 255usize;
	let num_blocks = dat_enc.chunks(block_size).count();
	let mut dat = Vec::with_capacity(dat_enc.len() - 8 * num_blocks);
	for chunk in dat_enc.chunks(block_size) {
		let dat_slice = from_erasure_block(chunk);
		dat.extend_from_slice(&dat_slice);
	}
	dat
}
