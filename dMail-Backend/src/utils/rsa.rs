/*! 用于rsa加密的函数 */

use rsa::{
    pkcs1::DecodeRsaPrivateKey, pkcs1::DecodeRsaPublicKey, Pkcs1v15Encrypt, PublicKey,
    RsaPrivateKey, RsaPublicKey,
};

use super::base64;

type PaddingType = Pkcs1v15Encrypt;

/** `get_pub_key_from_base64_pkcs1_pem` 从客户端发送的base64编码字符串中解析得到公钥
*/
// Rsa在Debug模式下相当的慢
pub fn get_pub_key_from_base64_pkcs1_pem(base64_str: String) -> Result<RsaPublicKey, ()> {
    let binary_public_key = match base64::decode(&base64_str) {
        Ok(key) => key,
        Err(_) => return Err(()),
    };

    return RsaPublicKey::from_pkcs1_der(&binary_public_key).map_err(|_| ());
}

/** `PubKeyHelper` 为rsa公钥类型封装的带base64的加密特征
*/
pub trait PubKeyHelper {
    /** `encrypt_to_base64` 将字符串通过rsa公钥加密并转成base64编码
     */
    fn encrypt_to_base64(&self, source: &[u8]) -> Result<String, ()>;
}

impl PubKeyHelper for RsaPublicKey {
    /** `encrypt_to_base64` 将字符串通过rsa公钥加密并转成base64编码
     */
    fn encrypt_to_base64(&self, data: &[u8]) -> Result<String, ()> {
        let mut rng = rand::thread_rng();

        let binary = match self.encrypt(&mut rng, PaddingType {}, &data) {
            Ok(data) => data,
            Err(_) => return Err(()),
        };

        return Ok(base64::encode(binary));
    }
}

/** `get_private_key_from_base64_pkcs1_pem` 从base64编码的字符串中解析得到rsa私钥，只在集成测试中使用该函数
*/
pub fn get_private_key_from_base64_pkcs1_pem(base64_str: String) -> Result<RsaPrivateKey, ()> {
    let binary_private_key = match base64::decode(&base64_str) {
        Ok(key) => key,
        Err(_) => return Err(()),
    };

    return RsaPrivateKey::from_pkcs1_der(&binary_private_key).map_err(|_| ());
}
