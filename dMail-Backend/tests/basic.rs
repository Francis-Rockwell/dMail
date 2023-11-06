mod common;
use common::testcase::TestCase;
use serial_test::serial;

#[test]
#[serial]
fn test_01_set_pub_key() {
    TestCase::read("set_pub_key_01").run();
    TestCase::read("set_pub_key_02").run();
}

#[test]
#[serial]
fn test_02_register() {
    TestCase::read("register_01_success").run();
    TestCase::read("register_02_email_registered").run();
    TestCase::read("register_03_password_format").run();
}

#[test]
#[serial]
fn test_03_login() {
    TestCase::read("login_01_success").run();
    TestCase::read("login_02_user_not_found").run();
    TestCase::read("login_03_logged").run();
    TestCase::read("login_04_password_error").run();
}

#[test]
#[serial]
fn test_04_user_setting() {
    TestCase::read("user_setting_01").run();
}

#[test]
#[serial]
fn test_05_update_user_info() {
    TestCase::read("update_user_info_01_name").run();
    TestCase::read("update_user_info_02_password").run();
}

#[test]
#[serial]
fn test_06_pull() {
    TestCase::read("pull_01_success").run();
}
