use std::sync::Mutex;
use tauri_app_lib::launcher::{claude_binary_path, find_claude, which_for_test};

// 序列化所有可能修改 PATH 的测试，避免 cargo test 默认多线程下互相污染。
static PATH_LOCK: Mutex<()> = Mutex::new(());

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

#[test]
fn test_which_prefers_exe_on_windows() {
    let _guard = PATH_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    // 准备临时目录，里面同时放 fake（sh 脚本 wrapper）和 fake.exe（PE 格式）
    let dir = tempfile::tempdir().expect("create tempdir");
    let sh = dir.path().join("fake");
    let exe = dir.path().join("fake.exe");
    std::fs::write(&sh, b"#!/bin/sh\necho fake\n").expect("write sh");
    std::fs::write(&exe, b"PE\x00\x00\x00\x00").expect("write exe");

    // 备份原 PATH
    let orig = std::env::var_os("PATH").unwrap_or_default();
    // 临时目录放最前面
    let new_path = std::env::join_paths(
        std::iter::once(dir.path().to_path_buf()).chain(std::env::split_paths(&orig)),
    )
    .expect("join path");

    // SAFETY: 测试用 Mutex 串行化、单线程下访问，无数据竞争
    unsafe { std::env::set_var("PATH", &new_path) };
    let result = which_for_test("fake");
    unsafe { std::env::set_var("PATH", &orig) };

    let p = result.expect("should find fake on PATH");
    let s = p.to_string_lossy();
    assert!(
        s.to_lowercase().ends_with("fake.exe"),
        "Windows 上 which 应该优先返回 .exe（PE 格式），实际: {}",
        s
    );
}

#[test]
fn test_which_falls_back_to_bat_when_no_exe() {
    // 验证次优顺序：没 .exe 时回退到 .bat / .cmd
    let _guard = PATH_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let dir = tempfile::tempdir().expect("create tempdir");
    let bat = dir.path().join("gizmo.bat");
    std::fs::write(&bat, b"@echo off").expect("write bat");

    let orig = std::env::var_os("PATH").unwrap_or_default();
    let new_path = std::env::join_paths(
        std::iter::once(dir.path().to_path_buf()).chain(std::env::split_paths(&orig)),
    )
    .expect("join path");

    unsafe { std::env::set_var("PATH", &new_path) };
    let result = which_for_test("gizmo");
    unsafe { std::env::set_var("PATH", &orig) };

    let p = result.expect("should find gizmo.bat on PATH");
    assert!(
        p.to_string_lossy().to_lowercase().ends_with("gizmo.bat"),
        "没有 .exe 时应该回退到 .bat，实际: {}",
        p.display()
    );
}

#[test]
fn test_which_returns_not_found_when_absent() {
    let _guard = PATH_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    // 用一个绝对不存在的名字
    let unique = format!("__definitely_not_on_path_{}__", std::process::id());
    let result = which_for_test(&unique);
    assert!(result.is_err(), "不存在时应该返回 Err，实际: {:?}", result);
}
