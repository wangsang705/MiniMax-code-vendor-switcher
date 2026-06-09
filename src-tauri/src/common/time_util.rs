use std::time::{SystemTime, UNIX_EPOCH};

/// 返回当前 Unix 时间戳（秒），作为数据库记录的时间字段
///
/// 所有模块统一调用此函数，确保时间格式一致。
pub fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 返回当前 Unix 时间戳（毫秒），用于备份文件名等需要高精度的场景
pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_ts_is_positive() {
        let ts = now_ts();
        assert!(ts > 1_700_000_000, "当前时间戳应大于 2023年底");
    }

    #[test]
    fn test_now_ms_is_larger_than_seconds() {
        let ms = now_ms();
        let ts = now_ts() as u128;
        assert!(ms >= ts * 1000, "毫秒戳应远大于秒戳");
    }
}
