use tauri_app_lib::launcher::{claude_binary_path, find_claude};

#[test]
fn test_claude_binary_path_format() {
    let p = claude_binary_path();
    let s = p.to_string_lossy();
    assert!(s.contains("claude") || s.contains("MiniMax-code"));
}

#[test]
fn test_find_claude_doesnt_panic() {
    // 找不到时返回 None，不 panic
    let _ = find_claude();
}
