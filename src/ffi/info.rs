use crate::backend::RealBackend;
use crate::model::{InfoGetReq, InfoListReq, InfoPostReq};
use crate::service;
use crate::utils::cjson;
use serde_json::Value;
use std::ffi::{c_char, c_int};

#[no_mangle]
pub extern "C" fn post_info_get(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: InfoGetReq = cjson::parse_json(json_in)?;
        service::info_get(&RealBackend, req)
    })
}

#[no_mangle]
pub extern "C" fn post_info_post(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: InfoPostReq = cjson::parse_json(json_in)?;
        service::info_post(&RealBackend, req)
    })
}

#[no_mangle]
pub extern "C" fn post_info_update(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: Value = cjson::parse_json(json_in)?;
        service::info_update(&RealBackend, &req)
    })
}

#[no_mangle]
pub extern "C" fn post_info_list(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    cjson::handle(json_out, || {
        let req: InfoListReq = cjson::parse_json(json_in)?;
        service::info_list(&RealBackend, req)
    })
}
