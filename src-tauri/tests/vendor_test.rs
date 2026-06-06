use tauri_app_lib::vendor::{presets, VendorPreset};

#[test]
fn test_presets_contain_expected_vendors() {
    let p = presets();
    let ids: Vec<&str> = p.iter().map(|v| v.id).collect();
    assert!(ids.contains(&"deepseek"));
    assert!(ids.contains(&"kimi"));
    assert!(ids.contains(&"zhipu"));
    assert!(ids.contains(&"qwen"));
    assert!(ids.contains(&"minimax"));
}

#[test]
fn test_preset_has_required_fields() {
    let p: Vec<VendorPreset> = presets();
    let ds = p.iter().find(|v| v.id == "deepseek").unwrap();
    assert_eq!(ds.api_base, "https://api.deepseek.com/anthropic");
    assert!(!ds.default_model.is_empty());
}
