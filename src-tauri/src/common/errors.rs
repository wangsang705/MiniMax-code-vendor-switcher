use serde::Serialize;

/// 应用统一错误类型
///
/// 所有 Tauri Command 返回 Result<T, AppError>，前端可根据 kind 字段
/// 区分错误类型（网络/权限/数据不存在），不再统一 alert()。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ── 数据层 ──
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("数据未找到: {0}")]
    NotFound(String),
    #[error("数据冲突: {0}")]
    Conflict(String),

    // ── Keyring 层 ──
    #[error("密钥环错误: {0}")]
    Keyring(String),
    #[error("缺少 API Key: {0}")]
    MissingApiKey(String),

    // ── 配置写入层 ──
    #[error("配置写入错误: {0}")]
    ConfigWrite(String),
    #[error("不支持的配置格式: {0}")]
    UnsupportedConfig(String),

    // ── 文件 IO ──
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML 错误: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("TOML 序列化错误: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("TOML 反序列化错误: {0}")]
    TomlDe(#[from] toml::de::Error),

    // ── 环境 ──
    #[error("无法找到用户目录")]
    NoHomeDir,
    #[error("不支持的平台: {0}")]
    UnsupportedPlatform(String),

    // ── 通用 ──
    #[error("内部错误: {0}")]
    Internal(String),
    #[error("参数错误: {0}")]
    InvalidArgument(String),
    #[error("{0}")]
    Custom(String),
}

/// 前端可读的错误响应结构
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub kind: &'static str,
    pub message: String,
}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}

impl AppError {
    /// 将错误转换为前端友好的结构，包含错误分类
    pub fn to_response(&self) -> ErrorResponse {
        let kind = match self {
            Self::Database(_) => "database",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
            Self::Keyring(_) => "keyring",
            Self::MissingApiKey(_) => "missing_api_key",
            Self::ConfigWrite(_) => "config_write",
            Self::UnsupportedConfig(_) => "unsupported_config",
            Self::Io(_) => "io",
            Self::Json(_) | Self::Yaml(_) | Self::TomlSer(_) | Self::TomlDe(_) => "serialization",
            Self::NoHomeDir => "no_home_dir",
            Self::UnsupportedPlatform(_) => "unsupported_platform",
            Self::Internal(_) => "internal",
            Self::InvalidArgument(_) => "invalid_argument",
            Self::Custom(_) => "error",
        };
        ErrorResponse {
            kind,
            message: self.to_string(),
        }
    }
}

/// 方便从 String/&str 构造 Custom 错误
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Custom(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Custom(s.to_string())
    }
}

/// 方便从 keyring::Error 转换
impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        AppError::Keyring(e.to_string())
    }
}
