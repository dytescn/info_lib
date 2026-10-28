use info_lib::{
    post_info_get, post_info_post, post_router_get, post_router_post, post_router_update,
};
use serde_json::Value;
use std::ffi::{c_char, CStr, CString};

fn call(
    f: unsafe extern "C" fn(*const c_char, *mut c_char) -> i32,
    input: Option<&CStr>,
) -> (i32, Value) {
    let mut out = vec![0u8; 65536];
    let ret = unsafe {
        f(
            input.map_or(std::ptr::null(), CStr::as_ptr),
            out.as_mut_ptr() as *mut c_char,
        )
    };
    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_str()
        .expect("utf-8 output");
    (ret, serde_json::from_str(text).expect("json output"))
}

#[test]
fn null_input_returns_400100() {
    let (ret, value) = call(post_router_get, None);
    assert_eq!(ret, 0);
    assert_eq!(value["code"], 400100);
}

#[test]
fn invalid_json_returns_400102() {
    let input = CString::new("{oops").unwrap();
    let (ret, value) = call(post_router_get, Some(&input));
    assert_eq!(ret, 0);
    assert_eq!(value["code"], 400102);
}

#[test]
fn invalid_utf8_returns_400101() {
    let input = CStr::from_bytes_with_nul(&[0xff, 0]).unwrap();
    let (ret, value) = call(post_router_get, Some(input));
    assert_eq!(ret, 0);
    assert_eq!(value["code"], 400101);
}

#[test]
fn missing_required_fields_are_rejected_before_backend() {
    let input = CString::new(r#"{"title":"x"}"#).unwrap();
    let (_, value) = call(post_router_post, Some(&input));
    assert_eq!(value["code"], 400102); // path / id 结构不符

    let input = CString::new(r#"{"id":1,"title":" ","path":"/x"}"#).unwrap();
    let (_, value) = call(post_router_post, Some(&input));
    assert_eq!(value["code"], 400103);

    let input = CString::new(r#"{}"#).unwrap();
    let (_, value) = call(post_info_post, Some(&input));
    assert_eq!(value["code"], 400102); // key 缺失

    let input = CString::new(r#"{}"#).unwrap();
    let (_, value) = call(post_info_get, Some(&input));
    assert_eq!(value["code"], 400104);
}

#[test]
fn update_requires_location() {
    let input = CString::new(r#"{"title":"x"}"#).unwrap();
    let (_, value) = call(post_router_update, Some(&input));
    assert_eq!(value["code"], 400104);
}
