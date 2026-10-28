mod real;

#[cfg(test)]
pub(crate) mod memory;

pub use real::RealBackend;

use crate::model::{Note, Route, Setting, Target};
use crate::utils::error::AppResult;
use serde_json::{Map, Value};

/// router / info 的存储后端。
///
/// 生产实现为 `RealBackend`（database.dll + libdesigndb.dll）；
/// 测试使用 `MemoryBackend`。
pub trait Backend {
    fn ensure_router(&self) -> AppResult<()>;
    fn ensure_setting(&self) -> AppResult<()>;

    fn router_find(&self, id: i64) -> AppResult<Option<Value>>;
    fn router_find_latest(&self, route: &Route) -> AppResult<Option<Value>>;
    fn router_insert(&self, route: &Route) -> AppResult<()>;
    fn router_update(&self, id: i64, fields: &Map<String, Value>) -> AppResult<Option<Value>>;
    fn router_list(
        &self,
        parent_id: Option<i64>,
        hide: Option<i64>,
        path: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Value>, i64)>;

    fn setting_find(&self, target: &Target) -> AppResult<Option<Value>>;
    fn setting_insert(&self, setting: &Setting) -> AppResult<()>;
    fn setting_update(
        &self,
        target: &Target,
        fields: &Map<String, Value>,
    ) -> AppResult<Option<Value>>;
    fn setting_list(
        &self,
        category: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Value>, i64)>;

    fn note_get(&self, key: &str, language: &str) -> AppResult<Option<Value>>;
    fn note_list(&self, language: &str) -> AppResult<Vec<Value>>;
    fn note_upsert(&self, note: &Note) -> AppResult<Value>;
}
