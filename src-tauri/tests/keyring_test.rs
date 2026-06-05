use tauri_app_lib::keyring_store::{set_key, get_key, delete_key};
use uuid;

const TEST_SERVICE: &str = "MiniMax-vendor-switcher-test";

#[test]
fn test_keyring_roundtrip() {
    let account = format!("test-account-{}", uuid::Uuid::new_v4());
    // 清理可能残留
    let _ = delete_key(TEST_SERVICE, &account);

    set_key(TEST_SERVICE, &account, "secret-token-123").unwrap();
    let got = get_key(TEST_SERVICE, &account).unwrap();
    assert_eq!(got, "secret-token-123");

    delete_key(TEST_SERVICE, &account).unwrap();
    assert!(get_key(TEST_SERVICE, &account).is_err());
}
