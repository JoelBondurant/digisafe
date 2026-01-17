use crate::storage::volatile;

use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	env, fs,
	io::{Read, Seek, SeekFrom, Write},
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
	fn get_nonce(&self) -> [u8; NONCE_SIZE] {
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
		compress(self.into_avro())
	}

	fn from_vec(dat: Vec<u8>) -> Self {
		Self::from_avro(decompress(dat))
	}

	fn from_outer(outer_db: OuterAvroDatabase, master_key: [u8; KEY_SIZE]) -> Self {
		let nonce = outer_db.get_nonce();
		let public_kv = outer_db.public_kv.clone();
		let db64 = to_base64(&encrypt(outer_db.into_avro(), master_key, nonce));
		Self { db64, public_kv }
	}

	fn into_outer(self, master_key: [u8; KEY_SIZE]) -> OuterAvroDatabase {
		let nonce = self.get_nonce();
		OuterAvroDatabase::from_avro(decrypt(from_base64(&self.db64), master_key, nonce).unwrap())
	}
}

impl OuterAvroDatabase {
	fn get_nonce(&self) -> [u8; NONCE_SIZE] {
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

	fn into_inner(self, master_key: [u8; KEY_SIZE]) -> InnerAvroDatabase {
		InnerAvroDatabase::from_outer(self, master_key)
	}

	fn from_inner(inner_db: InnerAvroDatabase, master_key: [u8; KEY_SIZE]) -> Self {
		inner_db.into_outer(master_key)
	}

	fn into_volatile(self, master_key: [u8; KEY_SIZE]) -> volatile::Database {
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

fn parse_nonce_from_kv(kv: &BTreeMap<String, String>) -> [u8; NONCE_SIZE] {
	let pre_nonce = kv.get("nonce").unwrap().parse::<u128>().unwrap();
	let mut nonce = [0u8; NONCE_SIZE];
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
		let dat = from_erasure_file(&db_name);
		let inner_db = InnerAvroDatabase::from_vec(dat);
		let digisalt: [u8; KEY_SIZE] = hex::decode(inner_db.public_kv.get("digisalt").unwrap())
			.unwrap()
			.try_into()
			.unwrap();
		let master_key = master_key_derivation(master_password, digisalt);
		let outer_db = OuterAvroDatabase::from_inner(inner_db, master_key);
		outer_db.into_volatile(master_key)
	} else {
		let mut digisalt = [0u8; KEY_SIZE];
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
	let dat = inner_db.into_vec();
	into_erasure_file(dat, &db_name);
	"Database saved.".to_string()
}

fn load_pepper() -> [u8; KEY_SIZE] {
	let pepper_output = Command::new("systemd-creds")
		.args(["decrypt", "--user", "--name=digipepper", &pepper_path()])
		.output()
		.unwrap();
	let pepper_hex = Zeroizing::new(pepper_output.stdout);
	let mut pepper = [0u8; KEY_SIZE];
	hex::decode_to_slice(
		std::str::from_utf8(&pepper_hex).unwrap().trim(),
		&mut pepper,
	)
	.unwrap();
	pepper
}

fn master_key_derivation(password: String, salt: [u8; KEY_SIZE]) -> [u8; KEY_SIZE] {
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
		.output_len(KEY_SIZE)
		.build()
		.unwrap();
	let main_hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, main_params);
	let mut main_hash = [0u8; KEY_SIZE];
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
	let post_hash: [u8; KEY_SIZE] = post_hasher.finalize().into();
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

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 24;

fn encrypt(msg: Vec<u8>, key: [u8; KEY_SIZE], nonce: [u8; NONCE_SIZE]) -> Vec<u8> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let cipher = XChaCha20Poly1305::new(&key.into());
	cipher.encrypt(&nonce.into(), &msg[..]).unwrap()
}

fn decrypt(msg_enc: Vec<u8>, key: [u8; KEY_SIZE], nonce: [u8; NONCE_SIZE]) -> Option<Vec<u8>> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let cipher = XChaCha20Poly1305::new(&key.into());
	cipher.decrypt(&nonce.into(), msg_enc.as_ref()).ok()
}

// Erasure section:
const DATA_SHARDS: usize = 8;
const PARITY_SHARDS: usize = 4;
const TOTAL_SHARDS: usize = DATA_SHARDS + PARITY_SHARDS;
const MIN_SHARD_SIZE: usize = 4096;

fn into_erasure_file(dat: Vec<u8>, db_name: &str) {
	use reed_solomon_erasure::galois_8::ReedSolomon;
	let original_len = dat.len() as u64;
	let raw_shard_size = dat.len().div_ceil(DATA_SHARDS);
	let shard_size = std::cmp::max(raw_shard_size, MIN_SHARD_SIZE);
	let mut padded_data = dat;
	padded_data.resize(shard_size * DATA_SHARDS, 0);
	let mut shards: Vec<Vec<u8>> = padded_data
		.chunks_exact(shard_size)
		.map(|c| c.to_vec())
		.collect();
	for _ in 0..PARITY_SHARDS {
		shards.push(vec![0; shard_size]);
	}
	let rs = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).unwrap();
	rs.encode(&mut shards).unwrap();
	let tmp_path = temp_path(db_name);
	let mut file = fs::File::create(&tmp_path).unwrap();
	for shard in shards {
		let hash = blake3::hash(&shard);
		file.write_all(&original_len.to_le_bytes()).unwrap();
		file.write_all(hash.as_bytes()).unwrap();
		file.write_all(&shard).unwrap();
	}
	file.sync_all().unwrap();
	mem::drop(file);
	let path = db_path(db_name);
	fs::rename(tmp_path, path).unwrap();
	padded_data.zeroize();
}

fn from_erasure_file(db_name: &str) -> Vec<u8> {
	use reed_solomon_erasure::galois_8::ReedSolomon;
	let mut file = fs::File::open(db_path(db_name)).unwrap();
	let file_len = file.metadata().unwrap().len();
	let chunk_size = (file_len as usize) / TOTAL_SHARDS;
	let header_size = 8 + 32;
	let shard_size = chunk_size - header_size;
	let mut original_len: Option<u64> = None;
	let mut shards: Vec<Option<Vec<u8>>> = Vec::new();
	for idx in 0..TOTAL_SHARDS {
		let mut meta_buf = [0u8; 8];
		let mut hash_buf = [0u8; 32];
		let mut data_buf = vec![0u8; shard_size];
		file.seek(SeekFrom::Start((idx * chunk_size) as u64))
			.unwrap();
		let success = file
			.read_exact(&mut meta_buf)
			.and_then(|_| file.read_exact(&mut hash_buf))
			.and_then(|_| file.read_exact(&mut data_buf));
		match success {
			Ok(_) => {
				if blake3::hash(&data_buf).as_bytes() == &hash_buf {
					original_len.get_or_insert(u64::from_le_bytes(meta_buf));
					shards.push(Some(data_buf));
				} else {
					shards.push(None);
				}
			}
			Err(_) => shards.push(None),
		}
	}
	let rs = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).unwrap();
	rs.reconstruct_data(&mut shards).unwrap();
	let mut recovered: Vec<u8> = shards
		.iter()
		.take(DATA_SHARDS)
		.filter_map(|shard| shard.as_ref())
		.flatten()
		.copied()
		.collect();
	recovered.truncate(original_len.unwrap() as usize);
	recovered
}
