mod engine;
mod game;

use wasm_bindgen::prelude::*;

use crate::engine::Engine;
use crate::game::MyGame;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // #[cfg(debug_assertions)]
    // console_error_panic_hook::set_once();

    let game = MyGame::new();
    Engine::launch(game);
    Ok(())
}
