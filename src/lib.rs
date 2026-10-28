//! info_lib：系统基础信息仓库。
//!
//! 提供 `router` / `info` 两组 FFI：
//! - `router_*`：`database/info.db` 的 `routers` 表；
//! - `info_*`：`database/info.db` 的 `setting` 表，`note` 字段写入
//!   `designdb/setting.setting_note`。

pub mod backend;
pub mod ffi;
pub mod model;
pub mod service;
pub mod utils;

pub use ffi::{info::*, router::*};
