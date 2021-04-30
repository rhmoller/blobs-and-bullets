pub mod error;
pub mod gamepad;
pub mod image_future;
pub mod json;
pub mod math;
pub mod preloader;
pub mod renderer;
pub mod tiled;

use crate::engine::image_future::ImageFuture;
use crate::engine::json::load_jsons;
use crate::engine::preloader::{Preloader, Resources};
use crate::engine::renderer::CanvasRenderer;
use crate::game::MyGame;

use crate::engine::gamepad::TwinStick;
use futures::future::join_all;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, GamepadButton, HtmlImageElement};

// mappings for PS4 Dual Shock in Firefox on Windows
// need to be changed depending on controller, browser and operating system
const LEFT_STICK_X_AXIS: u32 = 0;
const LEFT_STICK_Y_AXIS: u32 = 1;
const RIGHT_STICK_X_AXIS: u32 = 2;
const RIGHT_STICK_Y_AXIS: u32 = 3;
const RIGHT_TRIGGER_0: u32 = 7;

pub struct Engine {
    renderer: Rc<CanvasRenderer>,
    preloader: Preloader,
    tick: u64,
}

impl Engine {
    pub fn launch(mut game: MyGame) {
        spawn_local(async move {
            let mut engine = Engine::new();
            game.preload(&mut engine.preloader);

            let images = load_images(&engine.preloader.image_paths).await;
            let jsons = load_jsons(&engine.preloader.json_paths).await;
            game.init(Resources { images, jsons });
            engine.game_loop(game);
        });
    }

    pub fn new() -> Self {
        Engine {
            renderer: Rc::new(CanvasRenderer::new(960, 540)),
            preloader: Preloader::new(),
            tick: 0,
        }
    }

    fn game_loop(&self, mut game: MyGame) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let renderer = self.renderer.clone();
        let mut context = GameContext {
            window_width: self.renderer.width,
            window_height: self.renderer.height,
            gamepad_1: TwinStick::new(),
            gamepad_2: TwinStick::new(),
            tick: self.tick,
        };
        let window = window().unwrap();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if let Ok(gamepads) = window.navigator().get_gamepads() {
                if gamepads.length() > 0 {
                    if let Ok(gamepad) = gamepads.get(0).dyn_into::<web_sys::Gamepad>() {
                        context.gamepad_1.move_x_axis =
                            gamepad.axes().get(LEFT_STICK_X_AXIS).as_f64().unwrap();
                        context.gamepad_1.move_y_axis =
                            gamepad.axes().get(LEFT_STICK_Y_AXIS).as_f64().unwrap();

                        context.gamepad_1.aim_x_axis =
                            gamepad.axes().get(RIGHT_STICK_X_AXIS).as_f64().unwrap();
                        context.gamepad_1.aim_y_axis =
                            gamepad.axes().get(RIGHT_STICK_Y_AXIS).as_f64().unwrap();

                        context.gamepad_1.shoot = gamepad
                            .buttons()
                            .get(RIGHT_TRIGGER_0)
                            .dyn_into::<GamepadButton>()
                            .unwrap()
                            .pressed();
                    }
                }
                if gamepads.length() > 1 {
                    if let Ok(gamepad) = gamepads.get(1).dyn_into::<web_sys::Gamepad>() {
                        context.gamepad_2.move_x_axis =
                            gamepad.axes().get(LEFT_STICK_X_AXIS).as_f64().unwrap();
                        context.gamepad_2.move_y_axis =
                            gamepad.axes().get(LEFT_STICK_Y_AXIS).as_f64().unwrap();

                        context.gamepad_2.aim_x_axis =
                            gamepad.axes().get(RIGHT_STICK_X_AXIS).as_f64().unwrap();
                        context.gamepad_2.aim_y_axis =
                            gamepad.axes().get(RIGHT_STICK_Y_AXIS).as_f64().unwrap();

                        context.gamepad_2.shoot = gamepad
                            .buttons()
                            .get(RIGHT_TRIGGER_0)
                            .dyn_into::<GamepadButton>()
                            .unwrap()
                            .pressed();
                    }
                }
            }
            context.tick += 1;
            game.update(&context);
            game.render(renderer.as_ref(), &context);
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut() + 'static>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub struct GameContext {
    pub tick: u64,
    pub window_width: f64,
    pub window_height: f64,
    pub gamepad_1: TwinStick,
    pub gamepad_2: TwinStick,
}

async fn load_images(image_paths: &Vec<String>) -> HashMap<String, HtmlImageElement> {
    let image_futures: Vec<ImageFuture> = image_paths
        .iter()
        .map(|path| ImageFuture::new(path))
        .collect();

    let future: Vec<Result<HtmlImageElement, ()>> = join_all(image_futures).await;

    let images: HashMap<String, HtmlImageElement> = image_paths
        .iter()
        .zip(future.into_iter())
        .filter(|(_key, value)| (*value).is_ok())
        .map(|(key, value)| (key.clone(), value.unwrap()))
        .collect();
    images
}
