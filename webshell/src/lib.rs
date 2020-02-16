// ZQM:
extern crate zqm_engine;
use zqm_engine::{eval, types};

use wasm_bindgen::prelude::*;

use std::f64;

use web_sys::{self, console};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {

        let msg : JsValue = format!(
            "{} {} {} {} {} {}",
            event.key(),
            event.code(),
            event.shift_key(),
            event.ctrl_key(),
            alt_key(),
            meta_key()
        ).into();
        console::log_1(&msg);
    }) as Box<dyn FnMut(_)>);

    document.set_onkeypress(Some(closure.as_ref().unchecked_ref()));
    document.set_onkeydown(Some(closure.as_ref().unchecked_ref()));
    document.set_onkeyup(Some(closure.as_ref().unchecked_ref()));
    document.set_oninput(Some(closure.as_ref().unchecked_ref()));

    closure.forget();

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[wasm_bindgen]
pub fn canvas_test() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.set_fill_style(&"rgb(250,250,240)".into());
    context.fill_rect(0.0, 0.0, 666.0, 666.0);

    context.begin_path();

    // Draw the outer circle.
    context
        .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context
        .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context
        .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    context.stroke();

    Ok(())
}
