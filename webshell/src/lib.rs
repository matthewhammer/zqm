use wasm_bindgen::prelude::*;

use std::f64;

use wasm_bindgen::JsCast;
use web_sys::{self, console};

use std::cell::Cell;
use std::rc::Rc;

extern crate zqm_engine;
use zqm_engine::{
    eval, init, render,
    types::{
        event::{self, Event, KeyEventInfo},
        render::{Color, Elm, Elms, Fill, Pos, Rect},
    },
};

pub fn draw_elms(pos: &Pos, elms: &Elms) {
    // to do -- left this step outside of the recursion, for efficiency
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

    fn translate_color(c: &Color) -> JsValue {
        match c {
            &Color::RGB(r, g, b) => {
                let v: JsValue = format!("rgb({},{},{})", r as u8, g as u8, b as u8)
                    .as_str()
                    .into();
                v
            }
        }
    };

    fn draw_rect(context: &web_sys::CanvasRenderingContext2d, pos: &Pos, r: &Rect, f: &Fill) {
        match f {
            Fill::None => {
                // no-op.
            }
            Fill::Closed(c) => {
                let c: JsValue = translate_color(c);
                context.set_fill_style(&c);
                context.fill_rect(
                    (pos.x + r.pos.x) as f64,
                    (pos.y + r.pos.y) as f64,
                    r.dim.width as f64,
                    r.dim.height as f64,
                );
            }
            Fill::Open(c, 1) => {
                let c: JsValue = translate_color(c);
                context.set_stroke_style(&c);
                context.stroke_rect(
                    (pos.x + r.pos.x) as f64,
                    (pos.y + r.pos.y) as f64,
                    r.dim.width as f64,
                    r.dim.height as f64,
                );
            }
            Fill::Open(_, _) => unimplemented!(),
        }
    };

    for elm in elms.iter() {
        match &elm {
            &Elm::Node(node) => {
                let pos = Pos {
                    x: pos.x + node.rect.pos.x,
                    y: pos.y + node.rect.pos.y,
                };
                draw_rect(
                    &context,
                    &pos,
                    &Rect::new(0, 0, node.rect.dim.width, node.rect.dim.height),
                    &node.fill,
                );
                draw_elms(&pos, &node.children)
            }
            &Elm::Rect(r, f) => draw_rect(&context, pos, &r, &f),
        }
    }
}

pub fn console_log(m: String) {
    let message: JsValue = m.as_str().clone().into();
    console::log_1(&message);
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let mut state = init::init_state();
    draw_elms(&Pos { x: 0, y: 0 }, &eval::render_elms(&mut state).unwrap());
    let state_cell = Rc::new(Cell::new(state));
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut state: eval::State = state_cell.replace(init::init_state());
        let render_elms = {
            // translate each system event into zero, one or more in the engine's format.
            let events = match format!("{}", event.key()).as_str() {
                "Tab" | "Escape" | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | " " => {
                    vec![Event::KeyDown(KeyEventInfo {
                        key: event.key(),
                        alt: event.alt_key(),
                        ctrl: event.ctrl_key(),
                        shift: event.shift_key(),
                        meta: event.meta_key(),
                    })]
                }
                key => {
                    console_log(format!("unrecognized key: {}", key));
                    vec![]
                }
            };

            if false {
                console_log(format!("event key {} ==> events {:?}", event.key(), events));
            };
            // for each engine event, get commands from the engine,
            //   and run the commands in the engine, updating the state.
            for event in events.iter() {
                let commands = eval::commands_of_event(&mut state, event);
                match commands {
                    Ok(commands) => {
                        for command in commands.iter() {
                            let res = eval::command_eval(&mut state, command);
                            console_log(format!("eval({:?}) ==> {:?}", command, res))
                        }
                    }
                    Err(_) => {
                        // User is asking to escape; reset the state
                        console_log(format!("resetting state..."));
                        state = init::init_state();
                    }
                }
            }

            // get engine's render elements from updated state
            eval::render_elms(&mut state).unwrap()
        };
        // save updated state
        state_cell.set(state);
        // draw the engine elements onto the document's canvas element
        draw_elms(&Pos { x: 0, y: 0 }, &render_elms);
    }) as Box<dyn FnMut(_)>);

    let document = web_sys::window().unwrap().document().unwrap();
    document.set_onkeydown(Some(closure.as_ref().unchecked_ref()));
    //document.set_onkeypress(Some(closure.as_ref().unchecked_ref()));
    //document.set_onkeyup(Some(closure.as_ref().unchecked_ref()));
    //document.set_oninput(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    Ok(())
}
