use crate::utils::error::AppResult;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::ffi::{c_char, c_int, CStr};

pub const MAX_OUTPUT_LEN: usize = 65536;

#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI 入口，指针有效性由调用方保证
pub fn parse_json<T: DeserializeOwned>(json_in: *const c_char) -> AppResult<T> {
    if json_in.is_null() {
        return Err(crate::utils::error::AppError::new(
            400100,
            "json_in is null",
        ));
    }
    let body = unsafe { CStr::from_ptr(json_in) }
        .to_str()
        .map_err(|e| crate::utils::error::AppError::new(400101, format!("invalid utf-8: {e}")))?;
    if body.trim().is_empty() {
        return Ok(serde_json::from_str("{}").unwrap());
    }
    serde_json::from_str(body)
        .map_err(|e| crate::utils::error::AppError::new(400102, format!("invalid json: {e}")))
}

fn write_json_out(json_out: *mut c_char, value: Value) -> c_int {
    if json_out.is_null() {
        return -1;
    }
    let text = value.to_string();
    let bytes = text.as_bytes();
    if bytes.len() + 1 > MAX_OUTPUT_LEN {
        let fallback = r#"{"code":500900,"data":{},"msg":"response exceeds 64KB"}"#;
        unsafe {
            std::ptr::copy_nonoverlapping(fallback.as_ptr(), json_out as *mut u8, fallback.len());
            *json_out.add(fallback.len()) = 0;
        }
        return -2;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), json_out as *mut u8, bytes.len());
        *json_out.add(bytes.len()) = 0;
    }
    0
}

pub fn write_success(json_out: *mut c_char, data: Value) -> c_int {
    write_json_out(
        json_out,
        json!({ "code": 200, "data": data, "msg": "success" }),
    )
}

pub fn write_error(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    write_json_out(json_out, json!({ "code": code, "data": {}, "msg": msg }))
}

pub fn handle<F>(json_out: *mut c_char, f: F) -> c_int
where
    F: FnOnce() -> AppResult<Value>,
{
    match f() {
        Ok(data) => write_success(json_out, data),
        Err(e) => write_error(json_out, e.code, &e.msg),
    }
}
