use crate::backend::Backend;
use crate::model::{Note, Route, Setting, Target};
use crate::utils::db;
use crate::utils::error::{AppError, AppResult};
use crate::utils::time::now_iso;
use serde_json::{json, Map, Value};

const ROUTER_SCHEMA: &str = "id INTEGER PRIMARY KEY, title TEXT, icon TEXT, hide INTEGER, path TEXT, child TEXT, url TEXT, show INTEGER, parent_id INTEGER, level INTEGER";
const SETTING_SCHEMA: &str = "id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT NOT NULL UNIQUE, title TEXT, value TEXT, value_type TEXT DEFAULT 'string', category TEXT DEFAULT 'general', description TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP";
const NOTE_COLLECTION: &str = "setting_note";

pub struct RealBackend;

impl RealBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn sql_value(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(db::esc(s)),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "1" } else { "0" }.to_string()),
        Value::Null => Some("NULL".to_string()),
        _ => None,
    }
}

fn row_by(table: &str, where_sql: &str) -> AppResult<Option<Value>> {
    let rows = db::query(&format!("SELECT * FROM {table} WHERE {where_sql} LIMIT 1"))?;
    Ok(rows.into_iter().next())
}

fn opt_text(value: &Option<String>) -> String {
    value
        .as_deref()
        .map(db::esc)
        .unwrap_or_else(|| "NULL".to_string())
}

fn opt_int(value: Option<i64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "NULL".to_string())
}

fn set_clause(fields: &Map<String, Value>) -> AppResult<Vec<String>> {
    let mut sets = Vec::new();
    for (key, value) in fields {
        let sql = sql_value(value)
            .ok_or_else(|| AppError::new(400104, format!("unsupported value for {key}")))?;
        sets.push(format!("{} = {}", key, sql));
    }
    if sets.is_empty() {
        return Err(AppError::param("no fields to update"));
    }
    Ok(sets)
}

impl Backend for RealBackend {
    fn ensure_router(&self) -> AppResult<()> {
        db::create_table("routers", ROUTER_SCHEMA)
    }

    fn ensure_setting(&self) -> AppResult<()> {
        db::create_table("setting", SETTING_SCHEMA)
    }

    fn router_find(&self, id: i64) -> AppResult<Option<Value>> {
        row_by("routers", &format!("id = {id}"))
    }

    fn router_find_latest(&self, route: &Route) -> AppResult<Option<Value>> {
        let parent = route.parent_id.unwrap_or(0);
        let rows = db::query(&format!(
            "SELECT * FROM routers WHERE title = {} AND path = {} AND COALESCE(parent_id, 0) = {} ORDER BY id DESC LIMIT 1",
            db::esc(&route.title),
            db::esc(&route.path),
            parent
        ))?;
        Ok(rows.into_iter().next())
    }

    fn router_insert(&self, route: &Route) -> AppResult<()> {
        let mut columns = Vec::new();
        let mut values = Vec::new();
        if let Some(id) = route.id {
            columns.push("id");
            values.push(id.to_string());
        }
        columns.extend([
            "title",
            "icon",
            "hide",
            "path",
            "child",
            "url",
            "show",
            "parent_id",
            "level",
        ]);
        values.push(db::esc(&route.title));
        values.push(opt_text(&route.icon));
        values.push(opt_int(route.hide));
        values.push(db::esc(&route.path));
        values.push(opt_text(&route.child));
        values.push(opt_text(&route.url));
        values.push(opt_int(route.show));
        values.push(opt_int(route.parent_id));
        values.push(opt_int(route.level));
        db::insert(&format!(
            "INSERT INTO routers ({}) VALUES ({})",
            columns.join(", "),
            values.join(", ")
        ))
    }

    fn router_update(&self, id: i64, fields: &Map<String, Value>) -> AppResult<Option<Value>> {
        let sets = set_clause(fields)?;
        db::update(&format!(
            "UPDATE routers SET {} WHERE id = {}",
            sets.join(", "),
            id
        ))?;
        self.router_find(id)
    }

    fn router_list(
        &self,
        parent_id: Option<i64>,
        hide: Option<i64>,
        path: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Value>, i64)> {
        let mut parts = Vec::new();
        if let Some(v) = parent_id {
            parts.push(format!("COALESCE(parent_id, 0) = {v}"));
        }
        if let Some(v) = hide {
            parts.push(format!("hide = {v}"));
        }
        if let Some(v) = path.filter(|v| !v.is_empty()) {
            parts.push(format!("path = {}", db::esc(v)));
        }
        if let Some(v) = keyword.filter(|v| !v.is_empty()) {
            let p = db::esc(&format!("%{v}%"));
            parts.push(format!("(title LIKE {p} OR path LIKE {p})"));
        }
        let where_sql = if parts.is_empty() {
            "1 = 1".to_string()
        } else {
            parts.join(" AND ")
        };
        let total = db::query_count(&format!(
            "SELECT count(*) AS n FROM routers WHERE {where_sql}"
        ))?;
        let offset = (page - 1) * page_size;
        let list = db::query(&format!(
            "SELECT * FROM routers WHERE {where_sql} ORDER BY parent_id ASC, id ASC LIMIT {page_size} OFFSET {offset}"
        ))?;
        Ok((list, total))
    }

    fn setting_find(&self, target: &Target) -> AppResult<Option<Value>> {
        let where_sql = match target {
            Target::Id(id) => format!("id = {id}"),
            Target::Key(key) => format!("key = {}", db::esc(key)),
        };
        row_by("setting", &where_sql)
    }

    fn setting_insert(&self, setting: &Setting) -> AppResult<()> {
        db::insert(&format!(
            "INSERT INTO setting (key, title, value, value_type, category, description) VALUES ({}, {}, {}, {}, {}, {})",
            db::esc(&setting.key),
            opt_text(&setting.title),
            opt_text(&setting.value),
            db::esc(&setting.value_type),
            opt_text(&setting.category),
            opt_text(&setting.description)
        ))
    }

    fn setting_update(
        &self,
        target: &Target,
        fields: &Map<String, Value>,
    ) -> AppResult<Option<Value>> {
        let where_sql = match target {
            Target::Id(id) => format!("id = {id}"),
            Target::Key(key) => format!("key = {}", db::esc(key)),
        };
        let mut sets = set_clause(fields)?;
        sets.push(format!("updated_at = {}", db::now_sql()));
        db::update(&format!(
            "UPDATE setting SET {} WHERE {}",
            sets.join(", "),
            where_sql
        ))?;
        let lookup = match target {
            Target::Id(id) => Target::Id(*id),
            Target::Key(_) => fields
                .get("key")
                .and_then(Value::as_str)
                .map(|key| Target::Key(key.to_string()))
                .unwrap_or_else(|| target.clone()),
        };
        self.setting_find(&lookup)
    }

    fn setting_list(
        &self,
        category: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Value>, i64)> {
        let mut parts = Vec::new();
        if let Some(v) = category.filter(|v| !v.is_empty()) {
            parts.push(format!("category = {}", db::esc(v)));
        }
        if let Some(v) = keyword.filter(|v| !v.is_empty()) {
            let p = db::esc(&format!("%{v}%"));
            parts.push(format!(
                "(key LIKE {p} OR title LIKE {p} OR description LIKE {p})"
            ));
        }
        let where_sql = if parts.is_empty() {
            "1 = 1".to_string()
        } else {
            parts.join(" AND ")
        };
        let total = db::query_count(&format!(
            "SELECT count(*) AS n FROM setting WHERE {where_sql}"
        ))?;
        let offset = (page - 1) * page_size;
        let list = db::query(&format!(
            "SELECT * FROM setting WHERE {where_sql} ORDER BY category ASC, id ASC LIMIT {page_size} OFFSET {offset}"
        ))?;
        Ok((list, total))
    }

    fn note_get(&self, key: &str, language: &str) -> AppResult<Option<Value>> {
        let rows =
            db::designdb_query(NOTE_COLLECTION, json!({ "key": key, "language": language }))?;
        Ok(rows.into_iter().next())
    }

    fn note_list(&self, language: &str) -> AppResult<Vec<Value>> {
        db::designdb_query(NOTE_COLLECTION, json!({ "language": language }))
    }

    fn note_upsert(&self, note: &Note) -> AppResult<Value> {
        let filter = json!({ "key": note.key, "language": note.language });
        let existing = db::designdb_query(NOTE_COLLECTION, filter.clone())?;
        let now = now_iso();
        if existing.is_empty() {
            db::designdb_insert(
                NOTE_COLLECTION,
                json!({
                    "key": note.key,
                    "language": note.language,
                    "title": note.title,
                    "description": note.description,
                    "created_at": now,
                    "updated_at": now,
                }),
            )?;
        } else {
            db::designdb_update(
                NOTE_COLLECTION,
                filter,
                json!({
                    "title": note.title,
                    "description": note.description,
                    "updated_at": now,
                }),
            )?;
        }
        self.note_get(&note.key, &note.language)?
            .ok_or_else(|| AppError::new(500101, "note write failed"))
    }
}
