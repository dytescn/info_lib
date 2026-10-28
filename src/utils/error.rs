/// 业务错误：`code` 写入响应 JSON，`msg` 为可读说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub code: i32,
    pub msg: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(code: i32, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
        }
    }

    pub fn param(msg: impl Into<String>) -> Self {
        Self::new(400103, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(404100, msg)
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::new(409100, msg)
    }
}
