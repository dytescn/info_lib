use crate::backend::Backend;
use crate::model::{
    InfoGetReq, InfoListReq, InfoPostReq, Note, NoteReq, Route, RouterGetReq, RouterListReq,
    RouterPostReq, Setting, Target,
};
use crate::utils::error::{AppError, AppResult};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

const ROUTER_FIELDS: &[&str] = &[
    "title",
    "icon",
    "hide",
    "path",
    "child",
    "url",
    "show",
    "parent_id",
    "level",
];
const INFO_FIELDS: &[&str] = &[
    "key",
    "title",
    "value",
    "value_type",
    "category",
    "description",
];
const VALUE_TYPES: &[&str] = &[
    "string", "number", "boolean", "json", "array", "date", "enum", "path", "url", "color",
    "secret",
];

pub fn router_get(backend: &dyn Backend, req: RouterGetReq) -> AppResult<Value> {
    backend.ensure_router()?;
    backend
        .router_find(req.id)?
        .ok_or_else(|| AppError::not_found(format!("router {} not found", req.id)))
}

pub fn router_post(backend: &dyn Backend, req: RouterPostReq) -> AppResult<Value> {
    if req.id.is_some_and(|id| id <= 0) {
        return Err(AppError::new(400104, "id must be positive"));
    }
    require_text(&req.title, "title")?;
    require_text(&req.path, "path")?;
    let route = Route {
        id: req.id,
        title: req.title.trim().to_string(),
        icon: clean(req.icon),
        hide: req.hide,
        path: req.path.trim().to_string(),
        child: clean(req.child),
        url: clean(req.url),
        show: req.show,
        parent_id: req.parent_id,
        level: req.level,
    };
    backend.ensure_router()?;
    backend.router_insert(&route)?;
    let row = match route.id {
        Some(id) => backend.router_find(id)?,
        None => backend.router_find_latest(&route)?,
    }
    .ok_or_else(|| AppError::new(500100, "router write failed"))?;
    Ok(row)
}

pub fn router_update(backend: &dyn Backend, req: &Value) -> AppResult<Value> {
    let obj = req
        .as_object()
        .ok_or_else(|| AppError::new(400102, "request must be an object"))?;
    let id = obj
        .get("id")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .ok_or_else(|| AppError::new(400104, "id is required"))?;
    let fields = whitelisted(obj, ROUTER_FIELDS, &["id"])?;
    if fields.is_empty() {
        return Err(AppError::param("no fields to update"));
    }
    backend.ensure_router()?;
    backend
        .router_update(id, &fields)?
        .ok_or_else(|| AppError::not_found(format!("router {id} not found")))
}

pub fn router_list(backend: &dyn Backend, req: RouterListReq) -> AppResult<Value> {
    let (page, page_size) = page_params(req.page, req.page_size, 100, 500);
    backend.ensure_router()?;
    let (list, total) = backend.router_list(
        req.parent_id,
        req.hide,
        req.path.as_deref(),
        req.keyword.as_deref(),
        page,
        page_size,
    )?;
    Ok(json!({
        "list": list,
        "page": page,
        "page_size": page_size,
        "total": total,
    }))
}

pub fn info_get(backend: &dyn Backend, req: InfoGetReq) -> AppResult<Value> {
    let target = locate(req.id, req.key.as_deref())?;
    backend.ensure_setting()?;
    let row = backend
        .setting_find(&target)?
        .ok_or_else(|| AppError::not_found("setting not found"))?;
    with_note(backend, row)
}

pub fn info_post(backend: &dyn Backend, req: InfoPostReq) -> AppResult<Value> {
    require_text(&req.key, "key")?;
    let value_type = req.value_type.unwrap_or_else(|| "string".to_string());
    validate_value_type(&value_type)?;
    let note = req
        .note
        .map(|note| build_note(note, req.category.as_deref()))
        .transpose()?;
    let mut setting = Setting {
        key: req.key.trim().to_string(),
        title: clean(req.title),
        value: clean(req.value),
        value_type,
        category: clean(req.category),
        description: clean(req.description),
    };
    if setting.category.is_none() {
        if let Some(note) = &note {
            setting.category = Some(note.key.clone());
        }
    }
    backend.ensure_setting()?;
    backend.setting_insert(&setting)?;
    if let Some(note) = note {
        backend.note_upsert(&note)?;
    }
    let row = backend
        .setting_find(&Target::Key(setting.key.clone()))?
        .ok_or_else(|| AppError::new(500100, "setting write failed"))?;
    with_note(backend, row)
}

pub fn info_update(backend: &dyn Backend, req: &Value) -> AppResult<Value> {
    let obj = req
        .as_object()
        .ok_or_else(|| AppError::new(400102, "request must be an object"))?;
    let target = locate(
        obj.get("id").and_then(Value::as_i64),
        obj.get("key").and_then(Value::as_str),
    )?;
    let fields = whitelisted(obj, INFO_FIELDS, &["id", "key", "note"])?;
    let note_req = match obj.get("note") {
        None | Some(Value::Null) => None,
        Some(value) => Some(
            serde_json::from_value::<NoteReq>(value.clone())
                .map_err(|e| AppError::new(400102, format!("invalid note: {e}")))?,
        ),
    };
    if fields.is_empty() && note_req.is_none() {
        return Err(AppError::param("no fields to update"));
    }
    if let Some(Value::String(key)) = fields.get("key") {
        require_text(key, "key")?;
    }
    if let Some(Value::String(value_type)) = fields.get("value_type") {
        validate_value_type(value_type)?;
    }
    backend.ensure_setting()?;
    let row = if fields.is_empty() {
        backend
            .setting_find(&target)?
            .ok_or_else(|| AppError::not_found("setting not found"))?
    } else {
        backend
            .setting_update(&target, &fields)?
            .ok_or_else(|| AppError::not_found("setting not found"))?
    };
    if let Some(note) = note_req {
        let category = row
            .get("category")
            .and_then(Value::as_str)
            .map(str::to_string);
        let note = build_note(note, category.as_deref())?;
        backend.note_upsert(&note)?;
    }
    with_note(backend, row)
}

pub fn info_list(backend: &dyn Backend, req: InfoListReq) -> AppResult<Value> {
    let (page, page_size) = page_params(req.page, req.page_size, 50, 200);
    backend.ensure_setting()?;
    let (list, total) = backend.setting_list(
        req.category.as_deref(),
        req.keyword.as_deref(),
        page,
        page_size,
    )?;
    let notes = backend.note_list("default")?;
    let mut note_map: HashMap<String, Value> = HashMap::new();
    for note in notes {
        if let Some(key) = note.get("key").and_then(Value::as_str) {
            note_map.insert(key.to_string(), note);
        }
    }
    let mut rows = Vec::with_capacity(list.len());
    for mut row in list {
        let note = row
            .get("category")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .and_then(|category| note_map.get(category).cloned())
            .unwrap_or(Value::Null);
        if let Some(obj) = row.as_object_mut() {
            obj.insert("note".to_string(), note);
        }
        rows.push(row);
    }
    Ok(json!({
        "list": rows,
        "page": page,
        "page_size": page_size,
        "total": total,
    }))
}

fn with_note(backend: &dyn Backend, mut row: Value) -> AppResult<Value> {
    let category = row
        .get("category")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let note = match category {
        Some(category) => backend
            .note_get(&category, "default")?
            .unwrap_or(Value::Null),
        None => Value::Null,
    };
    if let Some(obj) = row.as_object_mut() {
        obj.insert("note".to_string(), note);
    }
    Ok(row)
}

fn build_note(req: NoteReq, fallback_category: Option<&str>) -> AppResult<Note> {
    let key = req
        .key
        .or_else(|| fallback_category.map(str::to_string))
        .unwrap_or_default();
    require_text(&key, "note.key")?;
    require_text(&req.title, "note.title")?;
    let language = req
        .language
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "default".to_string());
    Ok(Note {
        key: key.trim().to_string(),
        language: language.trim().to_string(),
        title: req.title.trim().to_string(),
        description: clean(req.description),
    })
}

fn locate(id: Option<i64>, key: Option<&str>) -> AppResult<Target> {
    if let Some(key) = key.filter(|v| !v.trim().is_empty()) {
        return Ok(Target::Key(key.trim().to_string()));
    }
    if let Some(id) = id.filter(|v| *v > 0) {
        return Ok(Target::Id(id));
    }
    Err(AppError::new(400104, "id or key is required"))
}

fn whitelisted(
    obj: &Map<String, Value>,
    allowed: &[&str],
    ignored: &[&str],
) -> AppResult<Map<String, Value>> {
    let mut fields = Map::new();
    for (key, value) in obj {
        if ignored.contains(&key.as_str()) {
            continue;
        }
        if !allowed.contains(&key.as_str()) {
            return Err(AppError::new(400104, format!("field not allowed: {key}")));
        }
        fields.insert(key.clone(), value.clone());
    }
    Ok(fields)
}

fn validate_value_type(value: &str) -> AppResult<()> {
    if VALUE_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::new(
            400104,
            format!("invalid value_type: {value}"),
        ))
    }
}

fn require_text(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::param(format!("{field} is required")))
    } else {
        Ok(())
    }
}

fn clean(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

fn page_params(
    page: Option<i64>,
    page_size: Option<i64>,
    default_size: i64,
    max_size: i64,
) -> (i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let size = page_size.unwrap_or(default_size).clamp(1, max_size);
    (page, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::memory::MemoryBackend;
    use crate::model::NoteReq;

    fn post_router(backend: &MemoryBackend, id: Option<i64>) -> Value {
        router_post(
            backend,
            RouterPostReq {
                id,
                title: "设置".to_string(),
                icon: Some("ic-sample".to_string()),
                hide: Some(1),
                path: "/setting".to_string(),
                child: None,
                url: None,
                show: None,
                parent_id: Some(0),
                level: Some(1),
            },
        )
        .expect("router post")
    }

    fn post_info(backend: &MemoryBackend, key: &str, note: Option<NoteReq>) -> Value {
        info_post(
            backend,
            InfoPostReq {
                key: key.to_string(),
                title: Some("界面语言".to_string()),
                value: Some("zh-CN".to_string()),
                value_type: Some("enum".to_string()),
                category: Some("base".to_string()),
                description: Some("zh-CN / en".to_string()),
                note,
            },
        )
        .expect("info post")
    }

    #[test]
    fn router_crud_and_list() {
        let backend = MemoryBackend::default();
        post_router(&backend, Some(500));

        let row = router_get(&backend, RouterGetReq { id: 500 }).expect("get");
        assert_eq!(row["title"], "设置");

        let row = router_update(&backend, &json!({ "id": 500, "hide": 0 })).expect("update");
        assert_eq!(row["hide"], 0);
        let row = router_get(&backend, RouterGetReq { id: 500 }).expect("get");
        assert_eq!(row["hide"], 0);

        let list = router_list(
            &backend,
            RouterListReq {
                parent_id: Some(0),
                ..Default::default()
            },
        )
        .expect("list");
        assert_eq!(list["total"], 1);
    }

    #[test]
    fn router_auto_id_and_conflict() {
        let backend = MemoryBackend::default();
        let row = post_router(&backend, None);
        assert_eq!(row["id"], 1);
        post_router(&backend, Some(500));
        let err = router_post(
            &backend,
            RouterPostReq {
                id: Some(500),
                title: "重复".to_string(),
                icon: None,
                hide: None,
                path: "/dup".to_string(),
                child: None,
                url: None,
                show: None,
                parent_id: None,
                level: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, 409100);
    }

    #[test]
    fn router_validation_errors() {
        let backend = MemoryBackend::default();
        assert_eq!(
            router_get(&backend, RouterGetReq { id: 999 })
                .unwrap_err()
                .code,
            404100
        );
        let err = router_update(&backend, &json!({ "id": 1, "unknown": 1 })).unwrap_err();
        assert_eq!(err.code, 400104);
        let err = router_post(
            &backend,
            RouterPostReq {
                id: Some(1),
                title: " ".to_string(),
                icon: None,
                hide: None,
                path: "/x".to_string(),
                child: None,
                url: None,
                show: None,
                parent_id: None,
                level: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, 400103);
    }

    #[test]
    fn info_crud_with_note() {
        let backend = MemoryBackend::default();
        let note = NoteReq {
            key: None,
            language: None,
            title: "基础".to_string(),
            description: Some("系统基础设置".to_string()),
        };
        let row = post_info(&backend, "language", Some(note.clone()));
        assert_eq!(row["note"]["key"], "base");
        assert_eq!(row["note"]["title"], "基础");

        let row = info_get(
            &backend,
            InfoGetReq {
                id: None,
                key: Some("language".to_string()),
            },
        )
        .expect("get");
        assert_eq!(row["note"]["description"], "系统基础设置");

        let list = info_list(
            &backend,
            InfoListReq {
                category: Some("base".to_string()),
                ..Default::default()
            },
        )
        .expect("list");
        assert_eq!(list["total"], 1);
        assert_eq!(list["list"][0]["note"]["title"], "基础");

        let row = info_update(
            &backend,
            &json!({
                "key": "language",
                "value": "en",
                "note": { "title": "基础设置" }
            }),
        )
        .expect("update");
        assert_eq!(row["value"], "en");
        assert_eq!(row["note"]["title"], "基础设置");
    }

    #[test]
    fn info_validation_and_conflict() {
        let backend = MemoryBackend::default();
        post_info(&backend, "language", None);
        let err = info_post(
            &backend,
            InfoPostReq {
                key: "language".to_string(),
                title: None,
                value: None,
                value_type: None,
                category: None,
                description: None,
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, 409100);

        let err = info_post(
            &backend,
            InfoPostReq {
                key: "bad".to_string(),
                title: None,
                value: None,
                value_type: Some("xml".to_string()),
                category: None,
                description: None,
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, 400104);

        let err = info_update(&backend, &json!({ "key": "language", "unknown": 1 })).unwrap_err();
        assert_eq!(err.code, 400104);

        let err = info_update(&backend, &json!({ "key": "language" })).unwrap_err();
        assert_eq!(err.code, 400103);

        let err = info_get(
            &backend,
            InfoGetReq {
                id: None,
                key: Some("missing".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(err.code, 404100);
    }

    #[test]
    fn info_note_key_defaults_to_category() {
        let backend = MemoryBackend::default();
        let note = NoteReq {
            key: None,
            language: None,
            title: "自定义分类".to_string(),
            description: None,
        };
        let row = post_info(&backend, "k1", Some(note));
        assert_eq!(row["note"]["key"], "base");
        assert_eq!(row["note"]["language"], "default");
    }
}
