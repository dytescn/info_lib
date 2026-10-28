# info_lib

System Basic Information Repository：`database/info.db` 的 `router` / `setting`
读写，以及 `designdb/setting.setting_note` 分类对象。

## 接口

| FFI-Symbol | 作用 |
| --- | --- |
| `router_get` / `router_list` | 读取单条 / 列表 |
| `router_post` / `router_update` | 新增 / 修改 |
| `info_get` / `info_list` | 读取单条 / 列表（合并 `note`） |
| `info_post` / `info_update` | 新增 / 修改；`note` 写入 designdb |

## 构建与测试

```bash
cargo build --release    # target/release/info_lib.dll
cargo test
cargo clippy --all-targets
```

## 目录

```text
src/
  ffi/router.rs / info.rs   C ABI 导出
  service.rs                校验、响应组装
  backend/real.rs           database.dll / libdesigndb.dll
  backend/memory.rs         测试后端
  model.rs                  请求 / 数据模型
  utils/cjson.rs            64 KiB JSON 输出
```
