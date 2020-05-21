use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::HtmlImageElement;

pub struct Preloader {
    pub image_paths: Vec<String>,
    pub json_paths: Vec<String>,
}

impl Preloader {
    pub fn new() -> Self {
        Preloader {
            image_paths: Vec::new(),
            json_paths: Vec::new(),
        }
    }

    pub fn load_image(&mut self, path: String) {
        self.image_paths.push(path);
    }

    pub fn load_json(&mut self, path: String) {
        self.json_paths.push(path);
    }
}

pub struct Resources {
    pub images: HashMap<String, HtmlImageElement>,
    pub jsons: HashMap<String, JsValue>,
}
