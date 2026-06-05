use tauri_app_lib::db::{self, init_db, VendorInstance};
use tempfile::tempdir;

#[test]
fn test_init_db_creates_schema() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = init_db(&db_path).unwrap();

    // 验证表已创建
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='vendors'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "vendors 表应该被创建");
}

#[test]
fn test_vendor_crud() {
    let dir = tempdir().unwrap();
    let conn = init_db(&dir.path().join("test.db")).unwrap();

    let v = VendorInstance {
        id: "v1".into(),
        preset_id: Some("deepseek".into()),
        name: "DeepSeek".into(),
        api_base: "https://api.deepseek.com".into(),
        model: "deepseek-chat".into(),
        keyring_key: "vendor:v1".into(),
        created_at: 100,
        updated_at: 100,
    };

    db::insert_vendor(&conn, &v).unwrap();
    let list = db::list_vendors(&conn).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "DeepSeek");

    let mut v2 = v.clone();
    v2.model = "deepseek-coder".into();
    v2.updated_at = 200;
    db::update_vendor(&conn, &v2).unwrap();
    let fetched = db::get_vendor(&conn, "v1").unwrap().unwrap();
    assert_eq!(fetched.model, "deepseek-coder");

    db::delete_vendor(&conn, "v1").unwrap();
    assert_eq!(db::list_vendors(&conn).unwrap().len(), 0);
}
