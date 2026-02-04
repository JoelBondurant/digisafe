use crate::storage::{
	database::{Database, InteriorDatabase},
	entry::MetaEntry,
	secret::SecretMemory,
};

use std::{
	env, fs,
	io::{Read, Seek, SeekFrom, Write},
	mem,
	path::PathBuf,
	process::Command,
	time::SystemTime,
};
use zeroize::{Zeroize, Zeroizing};

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

pub fn load(db_name: String, master_password: String) -> Database {
	let path = db_path(&db_name);
	if path.exists() {
		let db_outer_bin = from_erasure_file(&db_name);
		let db_outer = InteriorDatabase::deserialize(&db_outer_bin);
		let pre_nonce = db_outer
			.get_meta_entry("nonce")
			.unwrap()
			.get_value()
			.parse::<u128>()
			.unwrap();
		let mut nonce = [0u8; NONCE_SIZE];
		nonce[..16].copy_from_slice(&pre_nonce.to_le_bytes());
		let salt: [u8; 32] = from_base64(db_outer.get_meta_entry("salt").unwrap().get_value())
			.try_into()
			.unwrap();
		let master_key = master_key_derivation(master_password, salt);
		let db_encrypted_bin = from_base64(db_outer.get_meta_entry("db").unwrap().get_value());
		let db_compressed_bin = decrypt(db_encrypted_bin, &master_key, nonce).unwrap();
		let db_bin = decompress(db_compressed_bin);
		let db_inner = InteriorDatabase::deserialize(&db_bin);
		Database::old(master_key, db_inner)
	} else {
		let created_ts = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			.to_string();
		let mut salt = [0u8; KEY_SIZE];
		getrandom::fill(&mut salt).unwrap();
		let master_key = master_key_derivation(master_password, salt);
		let db = Database::new(master_key);
		db.set_meta_entry(MetaEntry::new("db_name", &db_name));
		db.set_meta_entry(MetaEntry::new("nonce", "0"));
		db.set_meta_entry(MetaEntry::new("salt", &to_base64(&salt)));
		db.set_meta_entry(MetaEntry::new("created_ts", &created_ts));
		db.set_meta_entry(MetaEntry::new("modified_ts", &created_ts));
		db
	}
}

pub fn save(db: Database) -> String {
	let db_name = db
		.get_meta_entry("db_name")
		.unwrap()
		.get_value()
		.to_string();
	let modified_ts = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.to_string();
	db.set_meta_entry(MetaEntry::new("modified_ts", &modified_ts));
	let num_nonce =
		db.get_meta_entry("nonce")
			.unwrap()
			.get_value()
			.parse::<u128>()
			.unwrap() + 1;
	let mut nonce = [0u8; NONCE_SIZE];
	nonce[..16].copy_from_slice(&num_nonce.to_le_bytes());
	db.set_meta_entry(MetaEntry::new("nonce", &num_nonce.to_string()));
	let inner_db = db.serialize();
	let inner_compressed = compress(inner_db.to_vec());
	let master_key = db.master_key.read().unwrap();
	let inner_encrypted = encrypt(inner_compressed, &master_key, nonce);
	let outer_db = db.meta_only();
	outer_db.set_meta_entry(MetaEntry::new("db", &to_base64(&inner_encrypted)));
	let db_bin = outer_db.serialize();
	into_erasure_file(db_bin.to_vec(), &db_name);
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

fn master_key_derivation(password: String, salt: [u8; KEY_SIZE]) -> SecretMemory {
	use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
	use sha3::{Digest, Sha3_256};
	let password = Zeroizing::new(password);
	let pepper = load_pepper();
	let mut pre_hasher = Sha3_256::new();
	pre_hasher.update(salt);
	pre_hasher.update(pepper);
	pre_hasher.update(password.as_bytes());
	let mut pre_hash = pre_hasher.finalize();
	let m_cost = if cfg!(debug_assertions) {
		2u32.pow(12)
	} else {
		2u32.pow(22)
	};
	let main_params = ParamsBuilder::new()
		.m_cost(m_cost)
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
	let mut post_hash: [u8; KEY_SIZE] = post_hasher.finalize().into();
	let mut pepper = pepper;
	pepper.zeroize();
	pre_hash.zeroize();
	main_hash.zeroize();
	let master_key = SecretMemory::new_pages(1).unwrap();
	master_key.write(0, &post_hash).unwrap();
	post_hash.zeroize();
	master_key
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

fn encrypt(msg: Vec<u8>, key: &SecretMemory, nonce: [u8; NONCE_SIZE]) -> Vec<u8> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let mut key: [u8; 32] = key.read().unwrap().to_vec().try_into().unwrap();
	let cipher = XChaCha20Poly1305::new(&key.into());
	key.zeroize();
	cipher.encrypt(&nonce.into(), &msg[..]).unwrap()
}

fn decrypt(msg_enc: Vec<u8>, key: &SecretMemory, nonce: [u8; NONCE_SIZE]) -> Option<Vec<u8>> {
	use chacha20poly1305::{
		aead::{Aead, KeyInit},
		XChaCha20Poly1305,
	};
	let mut key: [u8; 32] = key.read().unwrap().to_vec().try_into().unwrap();
	let cipher = XChaCha20Poly1305::new(&key.into());
	key.zeroize();
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
