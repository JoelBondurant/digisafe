use std::io::Write;
use zeroize::Zeroize;

pub fn master_key_derivation(password: &[u8], salt: &[u8]) -> [u8; 32] {
	use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
	use sha3::{Digest, Sha3_256};
	let mut pre_hasher = Sha3_256::new();
	pre_hasher.update(password);
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
		.hash_password_into(&pre_hash, salt, &mut main_hash)
		.unwrap();
	let mut post_hasher = Sha3_256::new();
	post_hasher.update(main_hash);
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

pub fn to_erasure_blocks(dat: Vec<u8>) -> Vec<u8> {
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
