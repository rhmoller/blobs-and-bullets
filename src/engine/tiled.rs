//! Support for maps created with https://www.mapeditor.org/
use serde::Deserialize;
use wasm_bindgen::JsValue;

#[derive(Debug, Deserialize)]
pub struct TileMap {
    pub width: u8,
    pub height: u8,
    pub layers: Vec<TileLayer>,
}

#[derive(Debug, Deserialize)]
pub struct TileLayer {
    pub width: u8,
    pub height: u8,
    pub data: Vec<u8>,
}

impl TileMap {
    pub fn new_from_json(json: &JsValue) -> TileMap {
        json.into_serde().unwrap()
    }
}
