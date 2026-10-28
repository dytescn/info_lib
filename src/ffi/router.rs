use crate::backend::RealBackend;
use crate::model::{RouterGetReq, RouterListReq, RouterPostReq};
use crate::service;
use crate::utils::cjson;
use serde_json::Value;
use std::ffi::{c_char, c_int};

#[no_mangle]
pub extern "C" fn post_router_get(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: RouterGetReq = cjson::parse_json(json_in)?;
        service::router_get(&RealBackend, req)
    })
}

#[no_mangle]
pub extern "C" fn post_router_post(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: RouterPostReq = cjson::parse_json(json_in)?;
        service::router_post(&RealBackend, req)
    })
}

#[no_mangle]
pub extern "C" fn post_router_update(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: Value = cjson::parse_json(json_in)?;
        service::router_update(&RealBackend, &req)
    })
}

#[no_mangle]
pub extern "C" fn post_router_list(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: RouterListReq = cjson::parse_json(json_in)?;
        service::router_list(&RealBackend, req)
    })
}
