use std::collections::HashMap;
use std::path::PathBuf;

use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{Aead, KeyInit};
use sha1::Sha1;
use sha3::Sha3_256;

pub struct AppDB {
    db_map: HashMap<String, String>,
    db_enc: String,
    db_id: String,
    password: [u8; 32],
    revision: String,
    version: String,
}

impl AppDB {

    pub fn new() -> Self {
        AppDB {
            db_map: HashMap::<String, String>::with_capacity(100),
            db_enc: "".to_owned(),
            db_id: "00000000".to_owned(),
            password: [0; 32],
            revision: "00000000".to_owned(),
            version: "00000000".to_owned(),
        }
    }

    pub fn get(&mut self, akey: &String) -> Option<String> {
        self.unlock();
        let raval = self.db_map.get(akey);
        let mut aval: Option<String> = None;
        if raval.is_some() {
            aval = Some(raval.unwrap().to_string());
        }
        self.lock();
        aval
    }

    pub fn set(&mut self, akey: String, aval: String) {
        use sha3::Digest;
        self.unlock();
        if akey.len() > 0 {
            if aval.len() > 0 {
                self.db_map.insert(akey, aval);
            } else {
                self.db_map.remove(&akey);
            }
        }
        let db_map_str = serde_json::to_string(&self.db_map).unwrap();
        let pre_prefix = self.version.to_string() + &self.db_id + &self.revision; // 8 + 8 + 8 = 24
        assert_eq!(pre_prefix.len(), 24);
        let hmac: [u8; 32] = Sha3_256::digest(base64::encode(self.password) + &pre_prefix + &db_map_str).try_into().unwrap();
        let nonce: [u8; 24] = hmac[..24].try_into().unwrap();
        let prefix = pre_prefix + &base64::encode(&nonce); // 24 + 32 = 56
        assert_eq!(prefix.len(), 56);
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
        PathBuf::from(format!("digisafe_{}.db", self.db_id))
    }

    fn db_path_hidden(&self) -> PathBuf {
        PathBuf::from(format!(".digisafe_{}.db", self.db_id))
    }

    fn db_path_archive(&self) -> PathBuf {
        let archive_root = PathBuf::from("archive").join(&self.db_id);
        let archive_file = PathBuf::from(format!("digisafe_{}.db", self.revision));
        archive_root.join(archive_file)
    }

    pub fn load(&mut self) -> String {
        if self.db_path().exists() {
            let rdb = std::fs::read_to_string(self.db_path());
            if rdb.is_ok() {
                let raw_db = rdb.unwrap();
                self.version = raw_db[..8].to_owned();
                self.db_id = raw_db[8..16].to_owned();
                self.revision = raw_db[16..24].to_owned();
                self.db_enc = raw_db.to_owned();
                self.unlock()
            } else {
                "load failure E1".into()
            }
        } else {
            "unlocked".into()
        }
    }

    pub fn save(&mut self) -> String {
        self.revision = format!("{:0>8}", self.revision.parse::<u16>().unwrap() + 1);
        self.set("".into(), "".into());
        let wr1 = std::fs::write(self.db_path_hidden(), &self.db_enc);
        if wr1.is_ok() {
            let wr2 = std::fs::rename(self.db_path_hidden(), &self.db_path());
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
        use sha3::Digest;
        if self.db_enc == "" {
            "unlocked".into()
        } else {
            let nonce: [u8; 24] = base64::decode(&self.db_enc[24..56]).unwrap().try_into().unwrap();
            let db_map_enc = &self.db_enc[56..];
            let db_map_str = AppDB::decrypt(db_map_enc.into(), self.password, &nonce);
            if db_map_str.is_some() {
                let db_map_str = db_map_str.unwrap();
                let pre_prefix = &self.db_enc[..24];
                let hmac: [u8; 32] = Sha3_256::digest(base64::encode(self.password) + &pre_prefix + &db_map_str).try_into().unwrap();
                let nonce_check: [u8; 24] = hmac[..24].try_into().unwrap();
                assert_eq!(nonce, nonce_check);
                let rdb: Result<HashMap<String, String>, _> = serde_json::from_str(&db_map_str);
                if rdb.is_ok() {
                    self.db_map.extend(rdb.unwrap().into_iter());
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
            hash_length: 32
        };
        let vhash = argon2::hash_raw(password.as_bytes(), salt, &config).unwrap();
        let hash: [u8; 32] = vhash.try_into().unwrap();
        hash
    }

    fn encrypt(raw_text: String, key: [u8; 32], nonce: &[u8; 24]) -> String {
        let cipher = XChaCha20Poly1305::new(&key.into());
        let cipher_text = cipher.encrypt(nonce.into(), raw_text.as_ref()).unwrap();
        base64::encode(cipher_text)
    }

    fn decrypt(enc_text: String, key: [u8; 32], nonce: &[u8; 24]) -> Option<String> {
        let cipher = XChaCha20Poly1305::new(&key.into());
        let blob = base64::decode(enc_text).unwrap();
        let vplain_text = cipher.decrypt(nonce.into(), blob.as_ref());
        if vplain_text.is_ok() {
            let plain_text = String::from_utf8(vplain_text.unwrap());
            if plain_text.is_ok() {
                Some(plain_text.unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn backup_db(&self) -> Result<String, reqwest::Error> {
        use sha1::Digest;
        let api_config: HashMap<String, String> = serde_json::from_str(&std::fs::read_to_string("/secrets/backblaze.json").unwrap()).unwrap();
        let api_key = base64::encode(format!("{}:{}", api_config["key_id"], api_config["app_key"]));
        let auth_url = "https://api.backblazeb2.com/b2api/v2/b2_authorize_account";
        let b2 = reqwest::blocking::Client::new();
        let auth_req = b2.get(auth_url).header("Authorization", format!("Basic {api_key}")).build().unwrap();
        let auth_resp = b2.execute(auth_req).unwrap().text().unwrap();
        let auth: HashMap<String, serde_json::Value> = serde_json::from_str(&auth_resp).unwrap();
        let auth_token = auth["authorizationToken"].clone().as_str().unwrap().to_string();
        let bucket_id = auth["allowed"]["bucketId"].clone().as_str().unwrap().to_string();
        let api_url = auth["apiUrl"].clone().as_str().unwrap().to_string();
        let upload_url_req = b2.post(format!("{api_url}/b2api/v2/b2_get_upload_url"))
            .body(format!("{{\"bucketId\":\"{bucket_id}\"}}"))
            .header("Authorization", format!("{auth_token}"))
            .build().unwrap();
        let upload_url_resp = b2.execute(upload_url_req).unwrap().text().unwrap();
        let upload_url_resp_map: HashMap<String, serde_json::Value> = serde_json::from_str(&upload_url_resp).unwrap();
        let upload_url = upload_url_resp_map["uploadUrl"].clone().as_str().unwrap().to_string();
        let upload_token = upload_url_resp_map["authorizationToken"].clone().as_str().unwrap().to_string();
        let mut sha1_hasher: Sha1 = Sha1::new();
        sha1_hasher.update(self.db_enc.as_bytes());
        let sha1_hash = hex::encode(sha1_hasher.finalize());
        let file_path = format!("{}/{}", self.db_id, self.db_path().file_name().unwrap().to_str().unwrap());
        let upload_req = b2.post(upload_url).body(self.db_enc.to_string())
            .header("Authorization", format!("{upload_token}"))
            .header("X-Bz-File-Name", file_path)
            .header("Content-Type", "text/plain")
            .header("X-Bz-Content-Sha1", sha1_hash)
            .header("X-Bz-Info-Author", "DigiSafe")
            .header("X-Bz-Server-Side-Encryption", "AES256")
            .build().unwrap();
        let upload_resp = b2.execute(upload_req).unwrap().text();
        upload_resp
    }

}
