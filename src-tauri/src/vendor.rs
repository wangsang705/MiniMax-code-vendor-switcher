pub struct VendorPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub api_base: &'static str,
    pub default_model: &'static str,
}

pub fn presets() -> Vec<VendorPreset> {
    vec![
        VendorPreset {
            id: "MiniMax",
            name: "MiniMax",
            api_base: "https://api.MiniMax.com",
            default_model: "MiniMax-M3",
        },
        VendorPreset {
            id: "deepseek",
            name: "DeepSeek",
            api_base: "https://api.deepseek.com",
            default_model: "deepseek-chat",
        },
        VendorPreset {
            id: "kimi",
            name: "Kimi (月之暗面)",
            api_base: "https://api.moonshot.cn/v1",
            default_model: "moonshot-v1-128k",
        },
        VendorPreset {
            id: "zhipu",
            name: "智谱 GLM",
            api_base: "https://open.bigmodel.cn/api/paas/v4",
            default_model: "glm-4-plus",
        },
        VendorPreset {
            id: "qwen",
            name: "Qwen (通义千问)",
            api_base: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            default_model: "qwen-plus",
        },
    ]
}
