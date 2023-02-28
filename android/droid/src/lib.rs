#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {

    use chacha20poly1305::ChaCha20Poly1305;
    use chacha20poly1305::aead::{Aead, KeyInit};
    use sha3::{Digest, Sha3_256};
    use jni::objects::{JClass};
    use jni::sys::jbyteArray;
    use jni::JNIEnv;


    #[no_mangle]
    pub unsafe extern "C" fn Java_com_digisafe_app_DroidKt_sha3(
        env: JNIEnv,
        _: JClass,
        arg: jbyteArray,
    ) -> jbyteArray {
        let hash = Sha3_256::digest(env.convert_byte_array(arg).unwrap().as_slice());
        env.byte_array_from_slice(hash.as_slice()).unwrap()
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_com_digisafe_app_DroidKt_encrypt(
        env: JNIEnv,
        _: JClass,
        key: jbyteArray,
        nonce: jbyteArray,
        arg: jbyteArray,
    ) -> jbyteArray {
        let key: [u8; 32] = env.convert_byte_array(key).unwrap().as_slice().try_into().unwrap();
        let nonce: [u8; 12] = env.convert_byte_array(nonce).unwrap().as_slice().try_into().unwrap();
        let arg: Vec<u8> = env.convert_byte_array(arg).unwrap();
        let cipher = ChaCha20Poly1305::new(&key.into());
        let cipher_txt = cipher.encrypt(&nonce.into(), arg.as_slice()).unwrap();
        env.byte_array_from_slice(cipher_txt.as_slice()).unwrap()
    }

}
