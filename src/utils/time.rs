use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// 当前 UTC 时间的 RFC3339 字符串。
pub fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
