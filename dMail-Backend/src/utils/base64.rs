/*! 用于base64加密的函数 */

use base64::{engine::general_purpose, Engine as _};

/** `encode` 将一个可以转成`[u8]`类型的输入值经过base64加密后的字符串输出
 */
pub fn encode<T>(binary: T) -> String
where
    T: AsRef<[u8]>,
{
    return general_purpose::STANDARD.encode(binary);
}

/** `decode` 输出将一个base64的字符串解码成`[u8]`的Result
 */
pub fn decode(str: &str) -> Result<Vec<u8>, ()> {
    return general_purpose::STANDARD.decode(str).map_err(|_| ());
}
