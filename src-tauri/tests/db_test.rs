use tauri_app_lib::db::{init_db, VendorInstance};
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
