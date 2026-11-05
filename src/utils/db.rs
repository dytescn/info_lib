use crate::utils::error::{AppError, AppResult};
use libloading::Library;
use serde_json::Value;
use std::ffi::{c_char, c_int, CStr, CString};
use std::sync::OnceLock;

const OUT_BUF_SIZE: usize = 65536;
#[cfg(windows)]
const DATABASE_LIB: &str = "./libary/database.dll";
#[cfg(target_os = "linux")]
const DATABASE_LIB: &str = "./libary/database.so";
#[cfg(target_os = "macos")]
const DATABASE_LIB: &str = "./libary/database.dylib";
#[cfg(windows)]
const DESIGNDB_LIB: &str = "./libary/libdesigndb.dll";
#[cfg(target_os = "linux")]
const DESIGNDB_LIB: &str = "./libary/libdesigndb.so";
#[cfg(target_os = "macos")]
const DESIGNDB_LIB: &str = "./libary/libdesigndb.dylib";

type FfiFn = unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int;

static DB_LIB: OnceLock<Library> = OnceLock::new();
static DESIGNDB: OnceLock<Library> = OnceLock::new();

fn load(slot: &'static OnceLock<Library>, path: &str, db_code: i32) -> AppResult<&'static Library> {
    if let Some(lib) = slot.get() {
        return Ok(lib);
    }
    let lib = unsafe { Library::new(path) }
        .map_err(|e| AppError::new(db_code, format!("load {path} failed: {e}")))?;
    let _ = slot.set(lib);
    Ok(slot.get().expect("library just set"))
}

fn call(lib: &Library, symbol: &[u8], body: Value, db_code: i32) -> AppResult<Value> {
    let func: libloading::Symbol<'_, FfiFn> = unsafe {
        lib.get(symbol)
            .map_err(|e| AppError::new(db_code, format!("dlsym failed: {e}")))?
    };
    let req = CString::new(body.to_string())
        .map_err(|_| AppError::new(400102, "request json contains \0"))?;
    let mut out = vec![0u8; OUT_BUF_SIZE];
    let ret = unsafe { func(req.as_ptr(), out.as_mut_ptr() as *mut c_char) };
    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_str()
        .map_err(|e| AppError::new(db_code, format!("invalid utf-8 response: {e}")))?;
    if text.is_empty() {
        return Err(AppError::new(
            db_code,
            format!("library returned empty (ret={ret})"),
        ));
    }
    let value: Value = serde_json::from_str(text)
        .map_err(|e| AppError::new(db_code, format!("invalid response: {e}")))?;
    let code = value.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if code != 200 && code != 0 {
        let msg = value
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("unknown error")
            .to_string();
        if msg.contains("UNIQUE constraint failed") {
            return Err(AppError::conflict(msg));
        }
        return Err(AppError::new(db_code, msg));
    }
    Ok(value)
}

fn database() -> AppResult<&'static Library> {
    load(&DB_LIB, DATABASE_LIB, 500100)
}

fn designdb() -> AppResult<&'static Library> {
    load(&DESIGNDB, DESIGNDB_LIB, 500101)
}

pub fn query(sql: &str) -> AppResult<Vec<Value>> {
    let value = call(
        database()?,
        b"post_db_query",
        serde_json::json!({ "path": "./info", "sql": sql }),
        500100,
    )?;
    Ok(value
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub fn query_count(sql: &str) -> AppResult<i64> {
    Ok(query(sql)?
        .first()
        .and_then(|row| row.get("n"))
        .and_then(Value::as_i64)
        .unwrap_or(0))
}

pub fn insert(sql: &str) -> AppResult<()> {
    call(
        database()?,
        b"post_db_insert",
        serde_json::json!({ "path": "./info", "sql": sql }),
        500100,
    )?;
    Ok(())
}

pub fn update(sql: &str) -> AppResult<()> {
    call(
        database()?,
        b"post_db_update",
        serde_json::json!({ "path": "./info", "sql": sql }),
        500100,
    )?;
    Ok(())
}

pub fn create_table(table: &str, schema: &str) -> AppResult<()> {
    call(
        database()?,
        b"post_db_create",
        serde_json::json!({ "path": "./info", "table": table, "schema": schema }),
        500100,
    )?;
    Ok(())
}

pub fn designdb_query(collection: &str, filter: Value) -> AppResult<Vec<Value>> {
    let value = call(
        designdb()?,
        b"post_db_query",
        serde_json::json!({
            "path": "setting",
            "collection": collection,
            "filter": filter,
        }),
        500101,
    )?;
    Ok(value
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub fn designdb_insert(collection: &str, doc: Value) -> AppResult<()> {
    call(
        designdb()?,
        b"post_db_insert",
        serde_json::json!({
            "path": "setting",
            "collection": collection,
            "doc": doc,
        }),
        500101,
    )?;
    Ok(())
}

pub fn designdb_update(collection: &str, filter: Value, update: Value) -> AppResult<()> {
    call(
        designdb()?,
        b"post_db_update",
        serde_json::json!({
            "path": "setting",
            "collection": collection,
            "filter": filter,
            // PoloDB 的 update 需要操作符，这里统一按字段更新处理
            "update": { "$set": update },
        }),
        500101,
    )?;
    Ok(())
}

pub fn esc(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn now_sql() -> &'static str {
    "datetime('now','localtime')"
}
