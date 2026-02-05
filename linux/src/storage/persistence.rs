use crate::storage::{
	database::{Database, InteriorDatabase},
	entry::MetaEntry,
	secret::SecretMemory,
};
use std::{
	collections::HashMap, env, fs, io::Write, mem, path::PathBuf, process::Command,
	time::SystemTime,
};
use zeroize::{Zeroize, Zeroizing};

const KEY_SIZE: usize = 32;
const SALT_SIZE: usize = 32;
const PEPPER_SIZE: usize = 32;
const NONCE_SIZE: usize = 24;

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

fn backblaze_path() -> String {
	let mut apath = base_path();
	apath.push("backblaze.cred");
	apath.to_str().unwrap().to_string()
}

pub async fn load(db_name: &str, master_password: String) -> Database {
	let path = db_path(db_name);
	let db_remote_bin_opt = download_db(db_name).await;
	match db_remote_bin_opt {
		None => {
			if !path.exists() {
				new_db(db_name, master_password)
			} else {
				db_from_vec(from_file(db_name), master_password)
			}
		}
		Some(db_remote_bin) => {
			if !path.exists() {
				db_from_vec(db_remote_bin, master_password)
			} else {
				let db_local_envelope = db_envelope_from_vec(from_file(db_name));
				let (nonce_local, salt) = parse_decryption_parameters(&db_local_envelope);
				let db_remote_envelope = db_envelope_from_vec(db_remote_bin);
				let (nonce_remote, _) = parse_decryption_parameters(&db_remote_envelope);
				let master_key = master_key_derivation(master_password, salt);
				let ts_local = db_local_envelope
					.get_meta_entry("modified_ts")
					.unwrap()
					.get_value()
					.parse::<u64>()
					.unwrap();
				let ts_remote = db_remote_envelope
					.get_meta_entry("modified_ts")
					.unwrap()
					.get_value()
					.parse::<u64>()
					.unwrap();
				if ts_local >= ts_remote {
					db_from_envelope(db_local_envelope, master_key, nonce_local)
				} else {
					db_from_envelope(db_remote_envelope, master_key, nonce_remote)
				}
			}
		}
	}
}

fn new_db(db_name: &str, master_password: String) -> Database {
	let created_ts = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.to_string();
	let mut salt = [0u8; SALT_SIZE];
	getrandom::fill(&mut salt).unwrap();
	let master_key = master_key_derivation(master_password, salt);
	let db = Database::new(master_key);
	db.set_meta_entry(MetaEntry::new("db_name", db_name));
	db.set_meta_entry(MetaEntry::new("nonce", "0"));
	db.set_meta_entry(MetaEntry::new("salt", &to_base64(&salt)));
	db.set_meta_entry(MetaEntry::new("created_ts", &created_ts));
	db.set_meta_entry(MetaEntry::new("modified_ts", &created_ts));
	db
}

fn db_envelope_from_vec(encoded: Vec<u8>) -> InteriorDatabase {
	InteriorDatabase::deserialize(&decode_erasure(&encoded))
}

fn parse_decryption_parameters(
	db_envelope: &InteriorDatabase,
) -> ([u8; NONCE_SIZE], [u8; SALT_SIZE]) {
	let pre_nonce = db_envelope
		.get_meta_entry("nonce")
		.unwrap()
		.get_value()
		.parse::<u128>()
		.unwrap();
	let mut nonce = [0u8; NONCE_SIZE];
	nonce[..16].copy_from_slice(&pre_nonce.to_le_bytes());
	let salt: [u8; SALT_SIZE] =
		from_base64(db_envelope.get_meta_entry("salt").unwrap().get_value())
			.try_into()
			.unwrap();
	(nonce, salt)
}

fn db_from_envelope(
	db_envelope: InteriorDatabase,
	master_key: SecretMemory,
	nonce: [u8; NONCE_SIZE],
) -> Database {
	let db_encrypted_bin = from_base64(db_envelope.get_meta_entry("db").unwrap().get_value());
	let db_compressed_bin = decrypt(db_encrypted_bin, &master_key, nonce).unwrap();
	let db_bin = decompress(db_compressed_bin);
	let db_core = InteriorDatabase::deserialize(&db_bin);
	Database::old(master_key, db_core)
}

fn db_from_vec(encoded: Vec<u8>, master_password: String) -> Database {
	let db_envelope = db_envelope_from_vec(encoded);
	let (nonce, salt) = parse_decryption_parameters(&db_envelope);
	let master_key = master_key_derivation(master_password, salt);
	db_from_envelope(db_envelope, master_key, nonce)
}

pub async fn save(db: &Database) -> String {
	let db_name = db
		.get_meta_entry("db_name")
		.unwrap()
		.get_value()
		.to_string();
	let db_bin = db_to_vec(db);
	to_file(&db_bin, &db_name);
	upload_db(&db_bin, &db_name).await;
	"Database saved.".to_string()
}

fn db_to_vec(db: &Database) -> Vec<u8> {
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
	let db_core = db.serialize();
	let db_core_compressed = compress(db_core.to_vec());
	let db_core_encrypted = encrypt(
		db_core_compressed,
		&db.clone_master_key().read().unwrap(),
		nonce,
	);
	let db_envelope = db.meta_only();
	db_envelope.set_meta_entry(MetaEntry::new("db", &to_base64(&db_core_encrypted)));
	let db_bin = db_envelope.serialize();
	encode_erasure(&db_bin)
}

fn load_pepper() -> [u8; PEPPER_SIZE] {
	let pepper_output = Command::new("systemd-creds")
		.args(["decrypt", "--user", "--name=digipepper", &pepper_path()])
		.output()
		.unwrap();
	let pepper_hex = Zeroizing::new(pepper_output.stdout);
	let mut pepper = [0u8; PEPPER_SIZE];
	hex::decode_to_slice(
		std::str::from_utf8(&pepper_hex).unwrap().trim(),
		&mut pepper,
	)
	.unwrap();
	pepper
}

fn master_key_derivation(password: String, salt: [u8; SALT_SIZE]) -> SecretMemory {
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

const ERASURE_DATA_SHARDS: usize = 8;
const ERASURE_PARITY_SHARDS: usize = 4;
const ERASURE_TOTAL_SHARDS: usize = ERASURE_DATA_SHARDS + ERASURE_PARITY_SHARDS;
const ERASURE_MIN_SHARD_SIZE: usize = 4096;

fn encode_erasure(data: &[u8]) -> Vec<u8> {
	use reed_solomon_erasure::galois_8::ReedSolomon;
	let original_len = data.len() as u64;
	let shard_size = std::cmp::max(
		data.len().div_ceil(ERASURE_DATA_SHARDS),
		ERASURE_MIN_SHARD_SIZE,
	);
	let mut padded = data.to_vec();
	padded.resize(shard_size * ERASURE_DATA_SHARDS, 0);
	let mut shards: Vec<Vec<u8>> = padded
		.chunks_exact(shard_size)
		.map(|chunk| chunk.to_vec())
		.collect();
	for _ in 0..ERASURE_PARITY_SHARDS {
		shards.push(vec![0; shard_size]);
	}
	let rs = ReedSolomon::new(ERASURE_DATA_SHARDS, ERASURE_PARITY_SHARDS).unwrap();
	rs.encode(&mut shards).unwrap();
	let mut output = Vec::new();
	for shard in shards {
		let hash = blake3::hash(&shard);
		output.extend_from_slice(&original_len.to_le_bytes());
		output.extend_from_slice(hash.as_bytes());
		output.extend_from_slice(&shard);
	}
	output
}

fn decode_erasure(encoded: &[u8]) -> Vec<u8> {
	use reed_solomon_erasure::galois_8::ReedSolomon;
	let chunk_size = encoded.len() / ERASURE_TOTAL_SHARDS;
	let mut original_len = None;
	let mut shards: Vec<Option<Vec<u8>>> = Vec::new();
	for chunk in encoded.chunks_exact(chunk_size) {
		let len = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
		let hash = &chunk[8..40];
		let data = &chunk[40..];
		if blake3::hash(data).as_bytes() == hash {
			original_len.get_or_insert(len);
			shards.push(Some(data.to_vec()));
		} else {
			shards.push(None);
		}
	}
	let rs = ReedSolomon::new(ERASURE_DATA_SHARDS, ERASURE_PARITY_SHARDS).unwrap();
	rs.reconstruct_data(&mut shards).unwrap();
	let mut recovered: Vec<u8> = shards
		.iter()
		.take(ERASURE_DATA_SHARDS)
		.filter_map(|shard| shard.as_ref())
		.flatten()
		.copied()
		.collect();
	recovered.truncate(original_len.unwrap() as usize);
	recovered
}

fn to_file(dat: &[u8], db_name: &str) {
	let tmp_path = temp_path(db_name);
	let mut file = fs::File::create(&tmp_path).unwrap();
	file.write_all(dat).unwrap();
	file.sync_all().unwrap();
	mem::drop(file);
	let path = db_path(db_name);
	fs::rename(&tmp_path, &path).unwrap();
	if let Some(parent) = path.parent() {
		let dir = fs::File::open(parent).unwrap();
		dir.sync_all().unwrap();
	}
}

fn from_file(db_name: &str) -> Vec<u8> {
	let path = db_path(db_name);
	fs::read(path).unwrap()
}

fn load_backblaze_creds() -> Vec<String> {
	String::from_utf8(
		Command::new("systemd-creds")
			.args(["decrypt", "--user", "--name=backblaze", &backblaze_path()])
			.output()
			.unwrap()
			.stdout,
	)
	.unwrap()
	.split("\0")
	.map(|x| x.to_owned())
	.collect()
}

async fn upload_db(db_bin: &[u8], db_name: &str) -> Option<String> {
	use sha1::{Digest, Sha1};
	let backblaze_creds = load_backblaze_creds();
	let api_key = to_base64(format!("{}:{}", backblaze_creds[1], backblaze_creds[2]).as_bytes());
	let auth_url = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";
	let b2 = reqwest::Client::new();
	let auth_req = b2
		.get(auth_url)
		.header("Authorization", format!("Basic {api_key}"))
		.build()
		.unwrap();
	let auth_resp = b2.execute(auth_req).await.unwrap().text().await.unwrap();
	let auth: HashMap<String, serde_json::Value> = serde_json::from_str(&auth_resp).unwrap();
	let auth_token = auth["authorizationToken"]
		.clone()
		.as_str()
		.unwrap()
		.to_string();
	let bucket_id = auth["allowed"]["bucketId"]
		.clone()
		.as_str()
		.unwrap()
		.to_string();
	let api_url = auth["apiUrl"].clone().as_str().unwrap().to_string();
	let upload_url_req = b2
		.post(format!("{api_url}/b2api/v2/b2_get_upload_url"))
		.body(format!("{{\"bucketId\":\"{bucket_id}\"}}"))
		.header("Authorization", auth_token)
		.build()
		.unwrap();
	let upload_url_resp = b2
		.execute(upload_url_req)
		.await
		.unwrap()
		.text()
		.await
		.unwrap();
	let upload_url_resp_map: HashMap<String, serde_json::Value> =
		serde_json::from_str(&upload_url_resp).unwrap();
	let upload_url = upload_url_resp_map["uploadUrl"]
		.clone()
		.as_str()
		.unwrap()
		.to_string();
	let upload_token = upload_url_resp_map["authorizationToken"]
		.clone()
		.as_str()
		.unwrap()
		.to_string();
	let mut sha1_hasher: Sha1 = Sha1::new();
	let body = to_base64(db_bin);
	sha1_hasher.update(body.as_bytes());
	let sha1_hash = hex::encode(sha1_hasher.finalize());
	let file_path = format!("{db_name}.digisafe");
	let upload_req = b2
		.post(upload_url)
		.body(body)
		.header("Authorization", upload_token)
		.header("X-Bz-File-Name", file_path)
		.header("Content-Type", "text/plain")
		.header("X-Bz-Content-Sha1", sha1_hash)
		.header("X-Bz-Info-Author", "DigiSafe")
		.header("X-Bz-Server-Side-Encryption", "AES256")
		.build()
		.unwrap();
	b2.execute(upload_req).await.unwrap().text().await.ok()
}

async fn download_db(db_name: &str) -> Option<Vec<u8>> {
	let backblaze_creds = load_backblaze_creds();
	let api_key = to_base64(format!("{}:{}", backblaze_creds[1], backblaze_creds[2]).as_bytes());
	let auth_url = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";
	let b2 = reqwest::Client::new();
	let auth_req = b2
		.get(auth_url)
		.header("Authorization", format!("Basic {api_key}"))
		.build()
		.unwrap();
	let auth_resp = b2.execute(auth_req).await.unwrap().text().await.unwrap();
	let auth: HashMap<String, serde_json::Value> = serde_json::from_str(&auth_resp).unwrap();
	let auth_token = auth["authorizationToken"]
		.clone()
		.as_str()
		.unwrap()
		.to_string();
	let download_url = auth["downloadUrl"].clone().as_str().unwrap().to_string();
	let download_req = b2
		.get(format!("{download_url}/file/digisafe/{db_name}.digisafe"))
		.header("Authorization", auth_token)
		.build()
		.unwrap();
	let download_resp = b2.execute(download_req).await.unwrap();
	if download_resp.status() == 200 {
		Some(from_base64(&download_resp.text().await.unwrap()))
	} else {
		None
	}
}
