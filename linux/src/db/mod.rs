use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::ChaCha20Poly1305;
use sha1::Sha1;
use sha2::Sha256;

pub struct AppDB {
	db_enc: String,
	db_id: String,
	db_map: HashMap<String, String>,
	password: [u8; 32],
	revision: String,
	version: String,
}

impl AppDB {
	pub fn new() -> Self {
		AppDB {
			db_enc: "".to_owned(),
			db_id: "00000000".to_owned(),
			db_map: HashMap::<String, String>::with_capacity(100),
			password: [0; 32],
			revision: "00000000".to_owned(),
			version: "00000000".to_owned(),
		}
	}

	pub fn get(&mut self, akey: &String) -> Option<String> {
		self.unlock();
		let raval = self.db_map.get(akey);
		let mut aval: Option<String> = None;
		if let Some(raval_val) = raval {
			aval = Some(raval_val.to_owned());
		}
		self.lock();
		aval
	}

	pub fn set(&mut self, akey: String, aval: String) {
		use sha2::Digest;
		self.unlock();
		if !akey.is_empty() {
			if !aval.is_empty() {
				self.db_map.insert(akey, aval);
			} else {
				self.db_map.remove(&akey);
			}
		}
		let db_map_str = serde_json::to_string(&self.db_map).unwrap();
		let pre_prefix = self.version.to_string() + &self.db_id + &self.revision; // 8 + 8 + 8 = 24
		assert_eq!(pre_prefix.len(), 24);
		let hmac_arg = base64::encode(self.password) + &pre_prefix + &db_map_str;
		let hmac_pre: [u8; 32] = Sha256::digest(hmac_arg).try_into().unwrap();
		let hmac: [u8; 32] = Sha256::digest(hmac_pre).try_into().unwrap();
		let nonce: [u8; 12] = hmac[..12].try_into().unwrap();
		let prefix = pre_prefix + &base64::encode(nonce); // 24 + 16 = 40
		assert_eq!(prefix.len(), 40);
		let db_str_enc = prefix + &AppDB::encrypt(db_map_str, self.password, &nonce);
		self.db_enc = db_str_enc;
		self.lock();
	}

	pub fn set_password(&mut self, raw_password: String) {
		self.password = AppDB::hash_password(raw_password);
	}

	pub fn set_db_id(&mut self, raw_dbid: String) {
		assert!(raw_dbid.len() <= 8);
		self.db_id = format!("{:0>8}", raw_dbid);
		self.set("".into(), "".into());
	}

	fn db_path(&self) -> PathBuf {
		let mut apath = env::home_dir().unwrap_or_default();
		apath.push(format!(".config/digisafe/digisafe_{}.db", self.db_id));
		fs::create_dir_all(apath.parent().unwrap()).ok();
		apath
	}

	fn db_path_hidden(&self) -> PathBuf {
		let mut apath = env::home_dir().unwrap_or_default();
		apath.push(format!(".config/digisafe/.digisafe_{}.db", self.db_id));
		fs::create_dir_all(apath.parent().unwrap()).ok();
		apath
	}

	fn db_path_archive(&self) -> PathBuf {
		let mut apath = env::home_dir().unwrap_or_default();
		apath.push("archive");
		apath.push(&self.db_id);
		apath.push(format!("digisafe_{}.db", self.revision));
		fs::create_dir_all(apath.parent().unwrap()).ok();
		apath
	}

	pub fn load(&mut self) -> String {
		if self.db_path().exists() {
			let rdb0 = fs::read_to_string(self.db_path());
			if let Ok(rdb) = rdb0 {
				let version = rdb[..8].to_owned();
				let db_id = rdb[8..16].to_owned();
				let revision = rdb[16..24].to_owned();
				let db_enc = rdb.to_owned();
				let rdb_bak = self.download_db();
				assert_eq!(self.db_id, db_id);
				assert_eq!(self.version, version);
				if !rdb_bak.is_empty() {
					let version_bak = rdb_bak[..8].to_owned();
					let db_id_bak = rdb_bak[8..16].to_owned();
					let revision_bak = rdb_bak[16..24].to_owned();
					let db_enc_bak = rdb_bak.to_owned();
					assert_eq!(self.db_id, db_id_bak);
					if version_bak == version && revision_bak > revision {
						self.db_enc = db_enc_bak;
						self.revision = revision_bak;
					} else {
						self.db_enc = db_enc;
						self.revision = revision;
					}
				} else {
					self.db_enc = db_enc;
					self.revision = revision;
				}
				self.unlock()
			} else {
				"load failure E1".into()
			}
		} else {
			let rdb_bak = self.download_db();
			if !rdb_bak.is_empty() {
				let version_bak = rdb_bak[..8].to_owned();
				let db_id_bak = rdb_bak[8..16].to_owned();
				let revision_bak = rdb_bak[16..24].to_owned();
				let db_enc_bak = rdb_bak.to_owned();
				assert_eq!(self.db_id, db_id_bak);
				if version_bak == self.version {
					self.db_enc = db_enc_bak;
					self.revision = revision_bak;
					self.unlock()
				} else {
					"unlocked".into()
				}
			} else {
				"unlocked".into()
			}
		}
	}

	pub fn save(&mut self) -> String {
		self.revision = format!("{:0>8}", self.revision.parse::<u16>().unwrap() + 1);
		self.set("".into(), "".into());
		let wr1 = std::fs::write(self.db_path_hidden(), &self.db_enc);
		if wr1.is_ok() {
			let wr2 = std::fs::rename(self.db_path_hidden(), self.db_path());
			if wr2.is_ok() {
				let wr3 = std::fs::create_dir_all(self.db_path_archive().parent().unwrap());
				if wr3.is_ok() {
					let wr4 = std::fs::copy(self.db_path(), self.db_path_archive());
					if wr4.is_ok() {
						let res = self.backup_db();
						if res.is_ok() {
							"saved".into()
						} else {
							"save failure E5".into()
						}
					} else {
						"save failure E4".into()
					}
				} else {
					"save failure E3".into()
				}
			} else {
				"save failure E2".into()
			}
		} else {
			"save failure E1".into()
		}
	}

	fn unlock(&mut self) -> String {
		use sha2::Digest;
		if self.db_enc.is_empty() {
			"unlocked".into()
		} else {
			let nonce: [u8; 12] = base64::decode(&self.db_enc[24..40])
				.unwrap()
				.try_into()
				.unwrap();
			let db_map_enc = &self.db_enc[40..];
			let odb_map_str = AppDB::decrypt(db_map_enc.into(), self.password, &nonce);
			if let Some(db_map_str) = odb_map_str {
				let pre_prefix = &self.db_enc[..24];
				let hmac_arg = base64::encode(self.password) + pre_prefix + &db_map_str;
				let hmac_pre: [u8; 32] = Sha256::digest(hmac_arg).try_into().unwrap();
				let hmac: [u8; 32] = Sha256::digest(hmac_pre).try_into().unwrap();
				let nonce_check: [u8; 12] = hmac[..12].try_into().unwrap();
				assert_eq!(nonce, nonce_check);
				let rrdb: Result<HashMap<String, String>, _> = serde_json::from_str(&db_map_str);
				if let Ok(rdb) = rrdb {
					self.db_map.extend(rdb);
					"unlocked".into()
				} else {
					"unlock failure E2".into()
				}
			} else {
				"unlock failure E1".into()
			}
		}
	}

	fn lock(&mut self) {
		self.db_map.clear();
	}

	fn hash_password(password: String) -> [u8; 32] {
		let salt = b"digisafe";
		let config = argon2::Config {
			variant: argon2::Variant::Argon2id,
			version: argon2::Version::Version13,
			mem_cost: 1048576,
			time_cost: 2,
			lanes: 4,
			thread_mode: argon2::ThreadMode::Parallel,
			secret: &[],
			ad: &[],
			hash_length: 32,
		};
		let vhash = argon2::hash_raw(password.as_bytes(), salt, &config).unwrap();
		let hash: [u8; 32] = vhash.try_into().unwrap();
		hash
	}

	fn encrypt(raw_text: String, key: [u8; 32], nonce: &[u8; 12]) -> String {
		let cipher = ChaCha20Poly1305::new(&key.into());
		let cipher_text = cipher.encrypt(nonce.into(), raw_text.as_ref()).unwrap();
		base64::encode(cipher_text)
	}

	fn decrypt(enc_text: String, key: [u8; 32], nonce: &[u8; 12]) -> Option<String> {
		let cipher = ChaCha20Poly1305::new(&key.into());
		let blob = base64::decode(enc_text).unwrap();
		let rvplain_text = cipher.decrypt(nonce.into(), blob.as_ref());
		if let Ok(vplain_text) = rvplain_text {
			String::from_utf8(vplain_text).ok()
		} else {
			None
		}
	}

	fn backup_db(&self) -> Result<String, reqwest::Error> {
		use sha1::Digest;
		let api_config: HashMap<String, String> =
			serde_json::from_str(&std::fs::read_to_string("/secrets/backblaze.json").unwrap())
				.unwrap();
		let api_key = base64::encode(format!(
			"{}:{}",
			api_config["key_id"], api_config["app_key"]
		));
		let auth_url = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";
		let b2 = reqwest::blocking::Client::new();
		let auth_req = b2
			.get(auth_url)
			.header("Authorization", format!("Basic {api_key}"))
			.build()
			.unwrap();
		let auth_resp = b2.execute(auth_req).unwrap().text().unwrap();
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
		let upload_url_resp = b2.execute(upload_url_req).unwrap().text().unwrap();
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
		sha1_hasher.update(self.db_enc.as_bytes());
		let sha1_hash = hex::encode(sha1_hasher.finalize());
		let file_path = format!("{}/{}", self.db_id, "digisafe.db");
		let upload_req = b2
			.post(upload_url)
			.body(self.db_enc.to_string())
			.header("Authorization", upload_token)
			.header("X-Bz-File-Name", file_path)
			.header("Content-Type", "text/plain")
			.header("X-Bz-Content-Sha1", sha1_hash)
			.header("X-Bz-Info-Author", "DigiSafe")
			.header("X-Bz-Server-Side-Encryption", "AES256")
			.build()
			.unwrap();
		b2.execute(upload_req).unwrap().text()
	}

	fn download_db(&self) -> String {
		let api_config: HashMap<String, String> =
			serde_json::from_str(&std::fs::read_to_string("/secrets/backblaze.json").unwrap())
				.unwrap();
		let api_key = base64::encode(format!(
			"{}:{}",
			api_config["key_id"], api_config["app_key"]
		));
		let auth_url = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";
		let b2 = reqwest::blocking::Client::new();
		let auth_req = b2
			.get(auth_url)
			.header("Authorization", format!("Basic {api_key}"))
			.build()
			.unwrap();
		let auth_resp = b2.execute(auth_req).unwrap().text().unwrap();
		let auth: HashMap<String, serde_json::Value> = serde_json::from_str(&auth_resp).unwrap();
		let auth_token = auth["authorizationToken"]
			.clone()
			.as_str()
			.unwrap()
			.to_string();
		let download_url = auth["downloadUrl"].clone().as_str().unwrap().to_string();
		let db_id = self.db_id.to_string();
		let download_req = b2
			.get(format!("{download_url}/file/digisafe/{db_id}/digisafe.db"))
			.header("Authorization", auth_token)
			.build()
			.unwrap();
		let download_resp = b2.execute(download_req).unwrap();
		if download_resp.status() == 200 {
			download_resp.text().unwrap()
		} else {
			"".to_owned()
		}
	}
}
