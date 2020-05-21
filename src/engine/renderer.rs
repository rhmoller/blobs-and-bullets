use crate::engine::tiled::TileMap;
use std::cmp::min;
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

pub struct CanvasRenderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    pub width: f64,
    pub height: f64,
}

impl CanvasRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let window = window().unwrap();
        let document = window.document().unwrap();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        document.body().unwrap().append_child(&canvas).unwrap();

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        ctx.set_image_smoothing_enabled(false);

        ctx.set_fill_style(&"#fff".into());

        CanvasRenderer {
            canvas,
            ctx,
            width: width as f64,
            height: height as f64,
        }
    }

    pub fn clear(&self) {
        self.ctx.set_fill_style(&"#000".into());
        self.ctx.fill_rect(
            0.,
            0.,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }

    pub fn draw_sprite(&self, image: &HtmlImageElement, idx: u8, x: f64, y: f64) {
        self.ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                8. * f64::from((idx) % 8),
                8. * f64::from((idx) / 8),
                8.,
                8.,
                x.round(),
                y.round(),
                16.,
                16.,
            )
            .unwrap();
    }

    pub fn draw_block(&self, image: &HtmlImageElement, idx: u8, x: f64, y: f64, w: u8, h: u8) {
        self.ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                8. * f64::from((idx) % 8),
                8. * f64::from((idx) / 8),
                8. * w as f64,
                8. * h as f64,
                x.round(),
                y.round(),
                16. * w as f64,
                16. * h as f64,
            )
            .unwrap();
    }

    pub fn draw_map(&self, map: &TileMap, tileset: &HtmlImageElement) {
        let tx_max = min(self.width as usize / 8, map.width as usize);
        let ty_max = min(self.height as usize / 8, map.height as usize);
        let data = &map.layers[0].data;
        for tx in 0..tx_max {
            for ty in 0..ty_max {
                self.ctx
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &tileset,
                        8. * f64::from((data[tx + ty * 60] - 1) % 8),
                        8. * f64::from((data[tx + ty * 60] - 1) / 8),
                        8.,
                        8.,
                        16. * (tx as f64),
                        16. * (ty as f64),
                        16.,
                        16.,
                    )
                    .unwrap();
            }
        }
    }

    pub fn draw_rect(&self, x: f64, y: f64, w: f64, h: f64) {
        self.ctx.set_fill_style(&"#0006".into());
        self.ctx.fill_rect(x, y, w, h);
    }

    pub fn draw_big_text(&self, image: &HtmlImageElement, x: f64, y: f64, text: &str) {
        let mut offset = 0.;

        text.chars().for_each(|c| {
            let cx = ((c as i8 - 65 + 32) % 8) * 16;
            let cy = ((c as i8 - 65 + 32) / 8) * 16;
            self.ctx
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    image,
                    cx as f64,
                    cy as f64,
                    16.,
                    16.,
                    x + offset,
                    y,
                    32.,
                    32.,
                )
                .unwrap();
            offset += 32.;
        })
    }

    pub fn draw_numbers(&self, image: &HtmlImageElement, x: f64, y: f64, text: &str) {
        let mut offset = 0.;

        text.chars().for_each(|c| {
            let cx = ((c as i8 - 48) % 18) * 3;
            let cy = ((c as i8 - 48) / 18) * 5;
            self.ctx
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    image,
                    cx as f64,
                    cy as f64,
                    3.,
                    5.,
                    x + offset,
                    y,
                    6.,
                    10.,
                )
                .unwrap();
            offset += 8.;
        });
    }

    pub fn draw_hearts(&self, numbers: &HtmlImageElement, x: f64, y: f64, count: i32) {
        let mut remainder = count;
        let mut i = 0;

        while remainder > 0 {
            let heart = if remainder > 5 { 0 } else { 5 - remainder };

            self.ctx
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    numbers,
                    0. + (heart as f64) * 6.,
                    5.,
                    6.,
                    5.,
                    x + 12. * i as f64,
                    y,
                    12.,
                    10.,
                )
                .unwrap();

            i += 1;
            remainder -= 5;
        }
    }

    pub fn draw_ammo(&self, numbers: &HtmlImageElement, x: f64, y: f64) {
        self.ctx
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                numbers, 0., 10., 6., 5., x, y, 12., 10.,
            )
            .unwrap();
    }
}
