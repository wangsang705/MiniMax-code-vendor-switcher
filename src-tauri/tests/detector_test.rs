// detector_test.rs — 测试工具检测逻辑
//
// 由 glm-5v-turbo 生成

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Clone, PartialEq)]
struct DetectedTool {
    name: String,
    path: PathBuf,
    version: Option<String>,
}

fn detect_tool(tool_name: &str, search_paths: &[PathBuf]) -> Option<DetectedTool> {
    for dir in search_paths {
        let candidate = dir.join(tool_name);
        if candidate.exists() && is_executable(&candidate) {
            let version = detect_version(&candidate, tool_name);
            return Some(DetectedTool {
                name: tool_name.to_string(),
                path: candidate,
                version,
            });
        }
        #[cfg(target_os = "windows")]
        {
            let candidate_exe = dir.join(format!("{}.exe", tool_name));
            if candidate_exe.exists() {
                let version = detect_version(&candidate_exe, tool_name);
                return Some(DetectedTool {
                    name: tool_name.to_string(),
                    path: candidate_exe,
                    version,
                });
            }
        }
    }
    None
}

fn detect_all_tools(search_paths: &[PathBuf]) -> Vec<DetectedTool> {
    let known_tools = ["claude", "codex", "aider", "opencode", "qwen", "kimichat", "grok"];
    let mut results = Vec::new();
    for tool in &known_tools {
        if let Some(detected) = detect_tool(tool, search_paths) {
            results.push(detected);
        }
    }
    results
}

fn detect_version(binary_path: &Path, tool_name: &str) -> Option<String> {
    let version_file = binary_path.parent().unwrap().join(format!("{}.version", tool_name));
    if version_file.exists() {
        let ver = fs::read_to_string(&version_file).unwrap();
        Some(ver.trim().to_string())
    } else {
        None
    }
}

fn is_executable(path: &Path) -> bool {
    // 目录不可执行，只有常规文件才可能是可执行工具
    if path.is_dir() {
        return false;
    }
    true
}

fn make_fake_binary(dir: &Path, name: &str, version: Option<&str>) -> PathBuf {
    let bin_path = dir.join(name);
    fs::write(&bin_path, format!("#!/bin/sh\necho 'I am {}'\n", name)).unwrap();
    if let Some(ver) = version {
        let ver_path = dir.join(format!("{}.version", name));
        fs::write(&ver_path, ver).unwrap();
    }
    bin_path
}

#[test]
fn test_detect_single_tool_found() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "claude", Some("1.0.17"));

    let result = detect_tool("claude", &[bin_dir.clone()]);
    assert!(result.is_some(), "应检测到 claude");
    let detected = result.unwrap();
    assert_eq!(detected.name, "claude");
    assert_eq!(detected.path, bin_dir.join("claude"));
    assert_eq!(detected.version, Some("1.0.17".to_string()));
}

#[test]
fn test_detect_tool_not_found() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let result = detect_tool("claude", &[bin_dir]);
    assert!(result.is_none(), "空目录中不应检测到任何工具");
}

#[test]
fn test_detect_tool_without_version() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "aider", None);
    let result = detect_tool("aider", &[bin_dir]);
    assert!(result.is_some());
    assert_eq!(result.unwrap().version, None);
}

#[test]
fn test_detect_uses_first_matching_path() {
    let tmp = TempDir::new().unwrap();
    let path_a = tmp.path().join("path_a");
    let path_b = tmp.path().join("path_b");
    fs::create_dir_all(&path_a).unwrap();
    fs::create_dir_all(&path_b).unwrap();
    make_fake_binary(&path_a, "claude", Some("1.0.0"));
    make_fake_binary(&path_b, "claude", Some("2.0.0"));

    let result = detect_tool("claude", &[path_a.clone(), path_b.clone()]);
    assert!(result.is_some());
    assert_eq!(result.unwrap().path, path_a.join("claude"));
}

#[test]
fn test_detect_falls_through_to_second_path() {
    let tmp = TempDir::new().unwrap();
    let path_a = tmp.path().join("path_a");
    let path_b = tmp.path().join("path_b");
    fs::create_dir_all(&path_a).unwrap();
    fs::create_dir_all(&path_b).unwrap();
    make_fake_binary(&path_b, "claude", Some("1.5.0"));

    let result = detect_tool("claude", &[path_a, path_b.clone()]);
    assert!(result.is_some());
    assert_eq!(result.unwrap().path, path_b.join("claude"));
}

#[test]
fn test_detect_with_empty_search_paths() {
    let result = detect_tool("claude", &[]);
    assert!(result.is_none(), "空搜索路径列表应返回 None");
}

#[test]
fn test_detect_all_tools_partial_install() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "claude", Some("1.0.17"));
    make_fake_binary(&bin_dir, "aider", Some("0.40.0"));
    make_fake_binary(&bin_dir, "codex", Some("0.1.0"));

    let results = detect_all_tools(&[bin_dir]);
    assert_eq!(results.len(), 3);
    let names: Vec<&str> = results.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"claude"));
    assert!(names.contains(&"aider"));
    assert!(names.contains(&"codex"));
}

#[test]
fn test_detect_all_tools_nothing_installed() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let results = detect_all_tools(&[bin_dir]);
    assert!(results.is_empty());
}

#[test]
fn test_detect_all_tools_from_multiple_dirs() {
    let tmp = TempDir::new().unwrap();
    let dir1 = tmp.path().join("dir1");
    let dir2 = tmp.path().join("dir2");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();
    make_fake_binary(&dir1, "claude", Some("1.0.0"));
    make_fake_binary(&dir2, "aider", Some("0.40.0"));

    let results = detect_all_tools(&[dir1, dir2]);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_version_parsing_standard_semver() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "claude", Some("1.0.17"));
    let detected = detect_tool("claude", &[bin_dir]).unwrap();
    assert_eq!(detected.version, Some("1.0.17".to_string()));
}

#[test]
fn test_version_parsing_with_prefix() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "codex", Some("v0.1.20250601"));
    let detected = detect_tool("codex", &[bin_dir]).unwrap();
    assert_eq!(detected.version, Some("v0.1.20250601".to_string()));
}

#[test]
fn test_version_parsing_with_whitespace() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "claude", None);
    fs::write(&bin_dir.join("claude.version"), "  1.0.17  \n").unwrap();
    let detected = detect_tool("claude", &[bin_dir]).unwrap();
    assert_eq!(detected.version, Some("1.0.17".to_string()));
}

#[test]
fn test_detect_ignores_directory_with_tool_name() {
    let tmp = TempDir::new().unwrap();
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir(bin_dir.join("claude")).unwrap();
    let result = detect_tool("claude", &[bin_dir]);
    assert!(result.is_none(), "同名目录不应被检测为工具");
}

#[test]
fn test_detect_handles_special_characters_in_path() {
    let tmp = TempDir::new().unwrap();
    let special_dir = tmp.path().join("工具 安装 目录");
    fs::create_dir_all(&special_dir).unwrap();
    make_fake_binary(&special_dir, "claude", Some("1.0.0"));
    let result = detect_tool("claude", &[special_dir]);
    assert!(result.is_some(), "路径含中文和空格时也应能检测到工具");
}

#[test]
fn test_detect_empty_dir_in_search_paths() {
    let tmp = TempDir::new().unwrap();
    let empty_dir = tmp.path().join("empty");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&empty_dir).unwrap();
    fs::create_dir_all(&bin_dir).unwrap();
    make_fake_binary(&bin_dir, "claude", Some("1.0.0"));
    let result = detect_tool("claude", &[empty_dir, bin_dir]);
    assert!(result.is_some());
}
