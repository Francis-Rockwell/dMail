use super::{errors, testcase::TestCase};
use assert_json_diff::{assert_json_matches_no_panic, CompareMode};
use dMail::user::user_session::protocol::ServerToClientMessage;
use errors as ERRORS;
use serde_json;
use std::panic;

impl TestCase {
    pub fn single_check(&mut self, expected: String, actual: &ServerToClientMessage) {
        let expected = serde_json::from_str::<ServerToClientMessage>(&expected);
        if expected.is_err() {
            self.kill_server();
            panic!("{}", ERRORS::STRING_TO_JSON_FAILED);
        }
        if let Err(error) = assert_json_matches_no_panic(
            &expected.unwrap(),
            &actual,
            assert_json_diff::Config::new(CompareMode::Inclusive),
        ) {
            println!("{:?}", error);
        }
    }

    pub fn multiple_check(&mut self, expecteds: Vec<String>, actuals: &Vec<ServerToClientMessage>) {
        let mut responses = vec![];

        for expected in expecteds {
            let expected = serde_json::from_str::<ServerToClientMessage>(&expected);
            if expected.is_err() {
                self.kill_server();
                panic!("{}", ERRORS::STRING_TO_JSON_FAILED);
            }
            responses.push(expected.unwrap());
        }

        if let Err(error) = assert_json_matches_no_panic(
            &responses,
            &actuals,
            assert_json_diff::Config::new(CompareMode::Inclusive),
        ) {
            println!("{:?}", error);
        }
    }
}
