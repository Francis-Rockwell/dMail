/*! 用于aes加密的函数 */

use aes_gcm::{aead::Aead, Aes128Gcm, Nonce};

use super::base64;

const IV: &[u8; 12] = b"dMailBackend";

// TODO : 创建NewType，缓存类型与buffer

/** `AesGcmHelper` 为Aes128Gcm密码封装的带base64的加密解密特征
 */
pub trait AesGcmHelper {
    /** `encrypt_with_default_nouce_to_base64` 把所给的字符串经过aes加密后，转为base64
     */
    fn encrypt_with_default_nouce_to_base64(&self, str: &str) -> Result<String, ()>;
    /** `decrypt_with_default_nouce_from_base64` 把base64编码下的经aes加密的信息转成可读字符串
     */
    fn decrypt_with_default_nouce_from_base64(&self, str: &str) -> Result<String, ()>;
}

impl AesGcmHelper for Aes128Gcm {
    /** `encrypt_with_default_nouce_to_base64` 把所给的字符串经过aes加密后，转为base64
     */
    fn encrypt_with_default_nouce_to_base64(&self, str: &str) -> Result<String, ()> {
        let nonce = Nonce::from_slice(IV);
        let binary = if let Ok(data) = self.encrypt(nonce, str.as_bytes()) {
            data
        } else {
            return Err(());
        };
        return Ok(base64::encode(binary));
    }

    /** `decrypt_with_default_nouce_from_base64` 把base64编码下的经aes加密的信息转成可读字符串
     */
    fn decrypt_with_default_nouce_from_base64(&self, str: &str) -> Result<String, ()> {
        let nonce = Nonce::from_slice(IV);

        let binary = if let Ok(data) = base64::decode(str) {
            data
        } else {
            return Err(());
        };

        let decoded_binary = if let Ok(data) = self.decrypt(nonce, binary.as_ref()) {
            data
        } else {
            return Err(());
        };

        return String::from_utf8(decoded_binary).map_err(|_| ());
    }
}
