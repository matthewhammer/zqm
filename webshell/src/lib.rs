// ZQM:
extern crate zqm_engine;
use zqm_engine::{eval, types::{render, event::{Event, KeyEventInfo}}};

use wasm_bindgen::prelude::*;

use std::f64;

use web_sys::{self, console};
//use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::rc::Rc;
use std::cell::Cell;

pub fn draw_elms(
    elms: &render::Elms,
) {
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

    // draw background; test canvas
    //context.set_fill_style(&"rgb(250,250,240)".into());
    //context.fill_rect(0.0, 0.0, 666.0, 666.0);

    fn translate_color(c:&render::Color) -> JsValue {
        match c {
            &render::Color::RGB(r, g, b) => {
                let v : JsValue =
                    format!("rgb({},{},{})",
                            r as u8,
                            g as u8,
                            b as u8
                    ).as_str().into();
                v
            }
        }
    };
    use zqm_engine::types::render::{Elm, Fill};
    for elm in elms.iter() {
        match &elm {
            &Elm::Node(_) => {
                unimplemented!()
            }
            &Elm::Rect(_r, Fill::None) => {
                // do nothing
            },
            &Elm::Rect(r, Fill::Closed(c)) => {
                let c : JsValue = translate_color(c);
                context.set_fill_style(&c);
                context.fill_rect(
                    r.pos.x as f64, r.pos.y as f64,
                    r.dim.width as f64,
                    r.dim.height as f64
                );
            },
            &Elm::Rect(r, Fill::Open(c, width)) => {
                assert_eq!(*width, 1);
                let c : JsValue = translate_color(c);
                context.set_stroke_style(&c);
                context.stroke_rect(
                    r.pos.x as f64, r.pos.y as f64,
                    r.dim.width as f64,
                    r.dim.height as f64
                );
            },
        }
    };
}

pub fn console_log(m:String) {
    let message : JsValue = m.as_str().clone().into();
    console::log_1(&message);
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let state_cell = Rc::new(Cell::new(eval::init_state()));
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut state : eval::State = state_cell.replace(eval::init_state());

        if false {
            let msg : JsValue = format!(
                "{} {} {} {} {} {}",
                event.key(),
                event.code(),
                event.shift_key(),
                event.ctrl_key(),
                event.alt_key(),
                event.meta_key()
            ).into();
            console::log_1(&msg);
        }

        // translate each system event into zero, one or more in the engine's format.
        // get commands from the engine, and run the commands for each event in the engine.
        let render_elms = {
            let events =
                match format!("{}", event.key()).as_str() {
                    "Escape" |
                    "ArrowUp" |
                    "ArrowDown" |
                    "ArrowLeft" |
                    "ArrowRight" |
                    " " =>
                    {
                        console_log(format!("recognized key: {}", event.key()));
                        vec![
                            Event::KeyDown(KeyEventInfo{
                                key: event.key(),
                                alt: event.alt_key(),
                                ctrl: event.ctrl_key(),
                                shift: event.shift_key(),
                                meta: event.meta_key()
                            }),
                        ]
                    },
                    key => {
                        console_log(format!("unrecognized key: {}", key));
                        vec![]
                    }
                };

            // for each event, get commands; for each command; eval it.
            console_log(format!("events: {:?}", events));
            for event in events.iter() {
                let commands = eval::commands_of_event(&mut state, event).unwrap();
                for command in commands.iter() {
                    eval::command_eval(&mut state, command).unwrap();
                }
            }
            eval::render_elms(&mut state).unwrap()
        };
        // save the new state, and draaw it.
        state_cell.set(state);
        draw_elms(&render_elms);

    }) as Box<dyn FnMut(_)>);
    //document.set_onkeypress(Some(closure.as_ref().unchecked_ref()));
    document.set_onkeydown(Some(closure.as_ref().unchecked_ref()));
    //document.set_onkeyup(Some(closure.as_ref().unchecked_ref()));
    //document.set_oninput(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(())
}
