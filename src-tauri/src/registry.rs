//! Windows 注册表检测模块
//!
//! 提供通过 Windows 注册表查找已安装桌面应用的能力。
//! 仅被 detector.rs 用于扫描检测，launcher.rs 不依赖此模块。
//!
//! 匹配策略：
//! 1. App Paths 下精确匹配子键名（如 "Codex.exe"）
//! 2. Uninstall 下匹配 DisplayName，且最终 exe 文件名必须匹配

use std::path::PathBuf;

/// 检查找到的 exe 路径的文件名是否与期望的 exe_name 匹配
/// 用于排除 "Codex++" 等不同软件的误匹配
/// 两边都去掉 .exe 后缀后比较文件名，所以 "Codex" 匹配 "Codex.exe" 但不匹配 "Codex++.exe"
fn exe_name_matches(found_path: &PathBuf, expected_exe: &str) -> bool {
    let file_stem = found_path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|n| n.to_lowercase())
        .unwrap_or_default();
    let expected_stem = expected_exe
        .trim_end_matches(".exe")
        .to_lowercase();
    file_stem == expected_stem
}

/// 通过 Windows 注册表检测已安装的桌面应用
///
/// - App Paths 键：子键名必须精确等于 exe_name（"Codex.exe" 不会匹配 "Codex++.exe"）
/// - Uninstall 键：匹配 DisplayName，且最终 exe 文件名必须与 exe_name 一致
#[cfg(windows)]
pub fn detect_desktop_via_registry(display_name_keyword: &str, exe_name: &str) -> Option<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    let exe_lower = exe_name.to_lowercase();

    let hives = [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER];
    let search_paths = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    for hive in &hives {
        for search_path in &search_paths {
            if let Ok(key) = RegKey::predef(*hive).open_subkey(search_path) {
                for subkey_name in key.enum_keys().flatten() {
                    let is_app_paths = search_path.contains("App Paths");

                    let matched = if is_app_paths {
                        // App Paths: 子键名可能是 "Codex" 或 "Codex.exe"，两种都匹配
                        let sub_lower = subkey_name.to_lowercase();
                        sub_lower == exe_lower || sub_lower == format!("{}.exe", exe_lower)
                    } else {
                        // Uninstall: 用 DisplayName + 文件名双重校验
                        if let Ok(subkey) = key.open_subkey(&subkey_name) {
                            let name_matches = subkey
                                .get_value::<String, _>("DisplayName")
                                .ok()
                                .map(|name| {
                                    let n = name.to_lowercase();
                                    n.contains(&display_name_keyword.to_lowercase())
                                })
                                .unwrap_or(false);
                            if !name_matches {
                                false
                            } else {
                                // DisplayName 匹配了，但还要能拿到 exe 路径才能算
                                true
                            }
                        } else {
                            false
                        }
                    };

                    if !matched {
                        continue;
                    }

                    if let Ok(subkey) = key.open_subkey(&subkey_name) {
                        // 收集所有候选路径，取第一个文件名匹配的
                        let mut candidates: Vec<PathBuf> = Vec::new();

                        // (Default) 值 —— App Paths 中的完整 exe 路径
                        if let Ok(val) = subkey.get_value::<String, _>("") {
                            let p = PathBuf::from(&val);
                            if p.is_file() {
                                candidates.push(p);
                            }
                        }

                        // InstallLocation + exe_name
                        if let Ok(loc) = subkey.get_value::<String, _>("InstallLocation") {
                            let dir = PathBuf::from(&loc);
                            let p1 = dir.join(exe_name);
                            if p1.is_file() {
                                candidates.push(p1);
                            }
                            let p2 = dir.join(format!("{}.exe", exe_name));
                            if p2.is_file() {
                                candidates.push(p2);
                            }
                        }

                        // DisplayIcon
                        if let Ok(icon) = subkey.get_value::<String, _>("DisplayIcon") {
                            let p = PathBuf::from(&icon);
                            if p.is_file() {
                                candidates.push(p);
                            }
                        }

                        // 在候选路径中取第一个文件名匹配期望 exe_name 的
                        for candidate in candidates {
                            if exe_name_matches(&candidate, exe_name) {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// 非 Windows 平台占位
#[cfg(not(windows))]
pub fn detect_desktop_via_registry(_display_name_keyword: &str, _exe_name: &str) -> Option<PathBuf> {
    None
}
