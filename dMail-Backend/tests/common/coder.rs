use super::testcase::TestCase;
use crate::common::errors;
use errors as ERRORS;

use dMail::utils::aes::AesGcmHelper;

impl TestCase {
    pub fn encode(&self, msg: String) -> String {
        if let Some(sym_key) = &self.sym_key {
            sym_key
                .encrypt_with_default_nouce_to_base64(&msg)
                .expect(ERRORS::ENCODE_FAILED)
        } else {
            msg
        }
    }

    pub fn decode(&self, msg: String) -> String {
        if let Some(sym_key) = &self.sym_key {
            sym_key
                .decrypt_with_default_nouce_from_base64(&msg)
                .expect(ERRORS::DECODE_FAILED)
        } else {
            msg
        }
    }
}
