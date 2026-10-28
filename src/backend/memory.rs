use crate::backend::Backend;
use crate::model::{Note, Route, Setting, Target};
use crate::utils::error::{AppError, AppResult};
use crate::utils::time::now_iso;
use serde_json::{json, Map, Value};
use std::sync::Mutex;

#[derive(Default)]
struct State {
    routers: Vec<Value>,
    settings: Vec<Value>,
    notes: Vec<Value>,
    next_router: i64,
    next_setting: i64,
}

pub struct MemoryBackend {
    state: Mutex<State>,
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                next_router: 1,
                next_setting: 1,
                ..State::default()
            }),
        }
    }
}

impl MemoryBackend {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("memory backend lock")
    }
}

impl Backend for MemoryBackend {
    fn ensure_router(&self) -> AppResult<()> {
        Ok(())
    }

    fn ensure_setting(&self) -> AppResult<()> {
        Ok(())
    }

    fn router_find(&self, id: i64) -> AppResult<Option<Value>> {
        Ok(self
            .lock()
            .routers
            .iter()
            .find(|row| row.get("id").and_then(Value::as_i64) == Some(id))
            .cloned())
    }

    fn router_find_latest(&self, route: &Route) -> AppResult<Option<Value>> {
        let state = self.lock();
        let mut matches: Vec<&Value> = state
            .routers
            .iter()
            .filter(|row| {
                row.get("title").and_then(Value::as_str) == Some(route.title.as_str())
                    && row.get("path").and_then(Value::as_str) == Some(route.path.as_str())
                    && row.get("parent_id").and_then(Value::as_i64).unwrap_or(0)
                        == route.parent_id.unwrap_or(0)
            })
            .collect();
        matches.sort_by_key(|row| row.get("id").and_then(Value::as_i64).unwrap_or(0));
        Ok(matches.last().map(|row| (*row).clone()))
    }

    fn router_insert(&self, route: &Route) -> AppResult<()> {
        let mut state = self.lock();
        let id = match route.id {
            Some(id) => {
                if state
                    .routers
                    .iter()
                    .any(|row| row.get("id").and_then(Value::as_i64) == Some(id))
                {
                    return Err(AppError::conflict(format!("router {id} already exists")));
                }
                id
            }
            None => {
                let id = state.next_router;
                state.next_router += 1;
                id
            }
        };
        state.routers.push(json!({
            "id": id,
            "title": route.title,
            "icon": route.icon,
            "hide": route.hide,
            "path": route.path,
            "child": route.child,
            "url": route.url,
            "show": route.show,
            "parent_id": route.parent_id,
            "level": route.level,
        }));
        Ok(())
    }

    fn router_update(&self, id: i64, fields: &Map<String, Value>) -> AppResult<Option<Value>> {
        let mut state = self.lock();
        let Some(row) = state
            .routers
            .iter_mut()
            .find(|row| row.get("id").and_then(Value::as_i64) == Some(id))
        else {
            return Err(AppError::not_found(format!("router {id} not found")));
        };
        let obj = row.as_object_mut().expect("router row is object");
        for (key, value) in fields {
            obj.insert(key.clone(), value.clone());
        }
        Ok(Some(row.clone()))
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
        let state = self.lock();
        let mut rows: Vec<Value> = state
            .routers
            .iter()
            .filter(|row| {
                parent_id
                    .is_none_or(|v| row.get("parent_id").and_then(Value::as_i64).unwrap_or(0) == v)
                    && hide.is_none_or(|v| row.get("hide").and_then(Value::as_i64) == Some(v))
                    && path.is_none_or(|v| {
                        v.is_empty() || row.get("path").and_then(Value::as_str) == Some(v)
                    })
                    && keyword.is_none_or(|v| {
                        if v.is_empty() {
                            return true;
                        }
                        let needle = v.to_lowercase();
                        let title = row
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_lowercase();
                        let path = row
                            .get("path")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_lowercase();
                        title.contains(&needle) || path.contains(&needle)
                    })
            })
            .cloned()
            .collect();
        rows.sort_by_key(|row| {
            (
                row.get("parent_id").and_then(Value::as_i64).unwrap_or(0),
                row.get("id").and_then(Value::as_i64).unwrap_or(0),
            )
        });
        let total = rows.len() as i64;
        let start = ((page - 1) * page_size).max(0) as usize;
        let list = rows
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((list, total))
    }

    fn setting_find(&self, target: &Target) -> AppResult<Option<Value>> {
        let state = self.lock();
        let row = state.settings.iter().find(|row| match target {
            Target::Id(id) => row.get("id").and_then(Value::as_i64) == Some(*id),
            Target::Key(key) => row.get("key").and_then(Value::as_str) == Some(key.as_str()),
        });
        Ok(row.cloned())
    }

    fn setting_insert(&self, setting: &Setting) -> AppResult<()> {
        let mut state = self.lock();
        if state
            .settings
            .iter()
            .any(|row| row.get("key").and_then(Value::as_str) == Some(setting.key.as_str()))
        {
            return Err(AppError::conflict(format!(
                "setting key {} already exists",
                setting.key
            )));
        }
        let id = state.next_setting;
        state.next_setting += 1;
        let now = now_iso();
        state.settings.push(json!({
            "id": id,
            "key": setting.key,
            "title": setting.title,
            "value": setting.value,
            "value_type": setting.value_type,
            "category": setting.category,
            "description": setting.description,
            "created_at": now,
            "updated_at": now,
        }));
        Ok(())
    }

    fn setting_update(
        &self,
        target: &Target,
        fields: &Map<String, Value>,
    ) -> AppResult<Option<Value>> {
        let mut state = self.lock();
        let Some(row) = state.settings.iter_mut().find(|row| match target {
            Target::Id(id) => row.get("id").and_then(Value::as_i64) == Some(*id),
            Target::Key(key) => row.get("key").and_then(Value::as_str) == Some(key.as_str()),
        }) else {
            return Err(AppError::not_found("setting not found"));
        };
        let obj = row.as_object_mut().expect("setting row is object");
        for (key, value) in fields {
            obj.insert(key.clone(), value.clone());
        }
        obj.insert("updated_at".to_string(), json!(now_iso()));
        Ok(Some(row.clone()))
    }

    fn setting_list(
        &self,
        category: Option<&str>,
        keyword: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Value>, i64)> {
        let state = self.lock();
        let mut rows: Vec<Value> = state
            .settings
            .iter()
            .filter(|row| {
                category.is_none_or(|v| {
                    v.is_empty() || row.get("category").and_then(Value::as_str) == Some(v)
                }) && keyword.is_none_or(|v| {
                    if v.is_empty() {
                        return true;
                    }
                    let needle = v.to_lowercase();
                    ["key", "title", "description"].iter().any(|field| {
                        row.get(*field)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(&needle)
                    })
                })
            })
            .cloned()
            .collect();
        rows.sort_by_key(|row| {
            (
                row.get("category")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                row.get("id").and_then(Value::as_i64).unwrap_or(0),
            )
        });
        let total = rows.len() as i64;
        let start = ((page - 1) * page_size).max(0) as usize;
        let list = rows
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((list, total))
    }

    fn note_get(&self, key: &str, language: &str) -> AppResult<Option<Value>> {
        let state = self.lock();
        Ok(state
            .notes
            .iter()
            .find(|row| {
                row.get("key").and_then(Value::as_str) == Some(key)
                    && row.get("language").and_then(Value::as_str) == Some(language)
            })
            .cloned())
    }

    fn note_list(&self, language: &str) -> AppResult<Vec<Value>> {
        let state = self.lock();
        Ok(state
            .notes
            .iter()
            .filter(|row| row.get("language").and_then(Value::as_str) == Some(language))
            .cloned()
            .collect())
    }

    fn note_upsert(&self, note: &Note) -> AppResult<Value> {
        let mut state = self.lock();
        let now = now_iso();
        if let Some(row) = state.notes.iter_mut().find(|row| {
            row.get("key").and_then(Value::as_str) == Some(note.key.as_str())
                && row.get("language").and_then(Value::as_str) == Some(note.language.as_str())
        }) {
            *row = json!({
                "key": note.key,
                "language": note.language,
                "title": note.title,
                "description": note.description,
                "created_at": row.get("created_at").cloned().unwrap_or_else(|| json!(now)),
                "updated_at": now,
            });
            return Ok(row.clone());
        }
        let row = json!({
            "key": note.key,
            "language": note.language,
            "title": note.title,
            "description": note.description,
            "created_at": now,
            "updated_at": now,
        });
        state.notes.push(row.clone());
        Ok(row)
    }
}
