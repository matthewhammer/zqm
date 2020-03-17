extern crate hashcons;
extern crate sdl2;
extern crate serde;

// Logging:
#[macro_use]
extern crate log;
extern crate env_logger;

// CLI: Representation and processing:
extern crate clap;
use clap::Shell;

extern crate structopt;
use structopt::StructOpt;

use sdl2::event::Event as SysEvent;
use sdl2::keyboard::Keycode;
use std::io;

// ZQM:
extern crate zqm_engine;
use zqm_engine::{
    candid, eval, init,
    types::{self, event, render},
};

/// zoom-quilt-maker
#[derive(StructOpt, Debug)]
#[structopt(name = "zqm", raw(setting = "clap::AppSettings::DeriveDisplayOrder"))]
struct CliOpt {
    /// Enable tracing -- the most verbose log.
    #[structopt(short = "t", long = "trace-log")]
    log_trace: bool,
    /// Enable logging for debugging.
    #[structopt(short = "d", long = "debug-log")]
    log_debug: bool,
    /// Disable most logging, if not explicitly enabled.
    #[structopt(short = "q", long = "quiet-log")]
    log_quiet: bool,
    #[structopt(subcommand)]
    command: CliCommand,
}

#[derive(StructOpt, Debug)]
enum CliCommand {
    #[structopt(name = "candid", about = "Start an interactive Candid session.")]
    Candid {
        replica_url: String,
        canister_id: String,
        did_file: String,
    },

    #[structopt(name = "start", about = "Start interactively.")]
    Start,

    #[structopt(name = "resume", about = "Resume last interaction.")]
    Resume,

    #[structopt(name = "version", about = "Display version.")]
    Version,

    #[structopt(
        name = "completions",
        about = "Generate shell scripts for zqm subcommand auto-completions."
    )]
    Completions { shell: Shell },
}

fn init_log(level_filter: log::LevelFilter) {
    use env_logger::{Builder, WriteStyle};
    let mut builder = Builder::new();
    builder
        .filter(None, level_filter)
        .write_style(WriteStyle::Always)
        .init();
}

use sdl2::render::{Canvas, RenderTarget};
pub fn draw_elms<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    pos: &render::Pos,
    dim: &render::Dim,
    fill: &render::Fill,
    elms: &render::Elms,
) -> Result<(), String> {
    fn translate_color(c: &render::Color) -> sdl2::pixels::Color {
        match c {
            &render::Color::RGB(r, g, b) => sdl2::pixels::Color::RGB(r as u8, g as u8, b as u8),
        }
    };
    fn translate_rect(pos: &render::Pos, r: &render::Rect) -> sdl2::rect::Rect {
        // todo -- clip the size of the rect dimension by the bound param
        sdl2::rect::Rect::new(
            (pos.x + r.pos.x) as i32,
            (pos.y + r.pos.y) as i32,
            r.dim.width as u32,
            r.dim.height as u32,
        )
    };
    fn draw_rect<T: RenderTarget>(
        canvas: &mut Canvas<T>,
        pos: &render::Pos,
        r: &render::Rect,
        f: &render::Fill,
    ) {
        match f {
            Fill::None => {
                // no-op.
            }
            Fill::Closed(c) => {
                let r = translate_rect(pos, r);
                let c = translate_color(c);
                canvas.set_draw_color(c);
                canvas.fill_rect(r).unwrap();
            }
            Fill::Open(c, 1) => {
                let r = translate_rect(pos, r);
                let c = translate_color(c);
                canvas.set_draw_color(c);
                canvas.draw_rect(r).unwrap();
            }
            Fill::Open(_c, _) => unimplemented!(),
        }
    };
    use zqm_engine::types::render::{Elm, Fill};
    draw_rect::<T>(
        canvas,
        &pos,
        &render::Rect::new(0, 0, dim.width, dim.height),
        fill,
    );
    for elm in elms.iter() {
        match &elm {
            &Elm::Node(node) => {
                let pos = render::Pos {
                    x: pos.x + node.rect.pos.x,
                    y: pos.y + node.rect.pos.y,
                };
                if false {
                    draw_rect::<T>(
                        canvas,
                        &pos,
                        &render::Rect::new(0, 0, node.rect.dim.width, node.rect.dim.height),
                        &node.fill,
                    );
                }
                draw_elms(canvas, &pos, &node.rect.dim, &node.fill, &node.children)?;
            }
            &Elm::Rect(r, f) => draw_rect(canvas, pos, r, f),
        }
    }
    Ok(())
}

fn translate_system_event(event: SysEvent) -> Option<event::Event> {
    match &event {
        SysEvent::Quit { .. }
        | SysEvent::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => Some(event::Event::Quit),
        SysEvent::KeyDown {
            keycode: Some(ref kc),
            ..
        } => {
            let key = match &kc {
                Keycode::Tab => "Tab".to_string(),
                Keycode::Space => " ".to_string(),
                Keycode::Return => "Enter".to_string(),
                Keycode::Left => "ArrowLeft".to_string(),
                Keycode::Right => "ArrowRight".to_string(),
                Keycode::Up => "ArrowUp".to_string(),
                Keycode::Down => "ArrowDown".to_string(),
                Keycode::Backspace => "Backspace".to_string(),
                keycode => format!("unrecognized({:?})", keycode),
            };
            let event = event::Event::KeyDown(event::KeyEventInfo {
                key: key,
                // to do -- translate modifier keys,
                alt: false,
                ctrl: false,
                meta: false,
                shift: false,
            });
            Some(event)
        }
        _ => None,
    }
}

pub fn do_event_loop(state: &mut types::lang::State) -> Result<(), String> {
    use sdl2::event::EventType;

    let pos = render::Pos { x: 0, y: 0 };
    let dim = render::Dim {
        width: 888,
        height: 666,
    };
    let fill = render::Fill::Closed(render::Color::RGB(0, 0, 0));

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("zoom-quilt-machine", dim.width as u32, dim.height as u32)
        .position_centered()
        //.resizable()
        //.input_grabbed()
        //.fullscreen()
        //.fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    info!("Using SDL_Renderer \"{}\"", canvas.info().name);

    {
        // draw initial frame, before waiting for any events
        let elms = eval::render_elms(state)?;
        draw_elms(&mut canvas, &pos, &dim, &fill, &elms)?;
        canvas.present();
        drop(elms);
    }

    let mut event_pump = sdl_context.event_pump()?;

    event_pump.disable_event(EventType::FingerUp);
    event_pump.disable_event(EventType::FingerDown);
    event_pump.disable_event(EventType::FingerMotion);
    event_pump.disable_event(EventType::MouseMotion);

    'running: loop {
        let event = translate_system_event(event_pump.wait_event());
        let event = match event {
            None => continue 'running,
            Some(event) => event,
        };
        match eval::commands_of_event(state, &event) {
            Ok(commands) => {
                for c in commands.iter() {
                    // note: we borrow the command here, possibly requiring some cloning when it is evaluated.
                    // todo -- we do nothing with the result; we should log it.
                    match eval::command_eval(state, c) {
                        Ok(()) => {}
                        Err(msg) => {
                            warn!("Command {:?} lead to an error:", c);
                            error!("{}", msg)
                        }
                    }
                }
                let elms = eval::render_elms(state)?;
                draw_elms(&mut canvas, &pos, &dim, &fill, &elms)?;
                canvas.present();
                drop(elms);
            }
            Err(()) => break 'running,
        }
    }
    Ok(())
}

fn main() {
    let cliopt = CliOpt::from_args();
    init_log(
        match (cliopt.log_trace, cliopt.log_debug, cliopt.log_quiet) {
            (true, _, _) => log::LevelFilter::Trace,
            (_, true, _) => log::LevelFilter::Debug,
            (_, _, true) => log::LevelFilter::Error,
            (_, _, _) => log::LevelFilter::Info,
        },
    );
    info!("Evaluating CLI command: {:?} ...", &cliopt.command);
    // - - - - - - - - - - -
    match cliopt.command {
        CliCommand::Candid {
            replica_url,
            canister_id,
            did_file,
        } => {
            use std::fs;
            let contents = fs::read_to_string(&did_file).expect("reading candid file");
            let ast = candid::parse_idl(&contents);
            //let menu = candid::menutype_of_idlprog_service(&ast);
            let mut state =
                candid::init(replica_url.as_str(), canister_id.as_str(), &ast)
                    .unwrap();
            do_event_loop(&mut state).unwrap();
            eval::save_state(&state);
        }
        CliCommand::Start => {
            let mut state = init::init_state();
            do_event_loop(&mut state).unwrap();
            eval::save_state(&state);
        }
        CliCommand::Resume => {
            let mut state = eval::load_state();
            do_event_loop(&mut state).unwrap();
            eval::save_state(&state);
        }
        CliCommand::Version => {
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");
            println!("{}", VERSION);
        }
        CliCommand::Completions { shell: s } => {
            // see also: https://clap.rs/effortless-auto-completion/
            //
            CliOpt::clap().gen_completions_to("zqm", s, &mut io::stdout());
            info!("done")
        }
    }
}
