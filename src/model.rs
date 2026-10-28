use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterGetReq {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterPostReq {
    #[serde(default)]
    pub id: Option<i64>,
    pub title: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub hide: Option<i64>,
    pub path: String,
    #[serde(default)]
    pub child: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub show: Option<i64>,
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub level: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouterListReq {
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub hide: Option<i64>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoGetReq {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteReq {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoPostReq {
    pub key: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub value_type: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub note: Option<NoteReq>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoListReq {
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub id: Option<i64>,
    pub title: String,
    pub icon: Option<String>,
    pub hide: Option<i64>,
    pub path: String,
    pub child: Option<String>,
    pub url: Option<String>,
    pub show: Option<i64>,
    pub parent_id: Option<i64>,
    pub level: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Setting {
    pub key: String,
    pub title: Option<String>,
    pub value: Option<String>,
    pub value_type: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub key: String,
    pub language: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Target {
    Id(i64),
    Key(String),
}

pub fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}
