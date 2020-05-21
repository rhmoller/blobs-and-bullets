use crate::engine::error::EngineError;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub async fn load_jsons(json_paths: &Vec<String>) -> HashMap<String, JsValue> {
    let mut jsons = HashMap::new();

    for path in json_paths.iter() {
        let result = load_json(path).await;
        if let Ok(value) = result {
            jsons.insert(path.clone(), value);
        }
    }

    jsons
}

pub async fn load_json(url: &str) -> Result<JsValue, EngineError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let json = JsFuture::from(resp.json()?).await?;
    Ok(json)
}
