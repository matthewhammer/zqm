// SDL: Keyboard/mouse input events, multi-media output abstractions:
extern crate hashcons;
extern crate sdl2;

// Logging:
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;

// CLI: Representation and processing:
extern crate clap;
use clap::Shell;

extern crate structopt;
use structopt::StructOpt;

use std::io;

// Unix OS:
//use std::process::Command as UnixCommand;

// ZQM:
extern crate zoom_quilt_maker;
use zoom_quilt_maker::{eval, types};

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
    #[structopt(name = "start", about = "Start interactively.")]
    Start,

    #[structopt(name = "resume", about = "Resume last interaction.")]
    Resume,

    #[structopt(name = "replay", about = "Replay last interaction.")]
    Replay,

    #[structopt(
        name = "history",
        about = "Interact with history, the list of all prior interactions."
    )]
    History,

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

pub fn do_event_loop(state: &mut types::State) -> Result<(), String> {
    use sdl2::event::EventType;
    //use sdl2::keyboard::Keycode;

    let grid_size = (16, 16);
    let zoom = 32u32;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(
            "zoom-quilt-maker",
            grid_size.0 * zoom + 1,
            grid_size.1 * zoom + 1,
        )
        .position_centered()
        .resizable()
        //.input_grabbed()
        .fullscreen()
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    info!("Using SDL_Renderer \"{}\"", canvas.info().name);

    let mut event_pump = sdl_context.event_pump()?;

    event_pump.disable_event(EventType::FingerUp);
    event_pump.disable_event(EventType::FingerDown);
    event_pump.disable_event(EventType::FingerMotion);
    event_pump.disable_event(EventType::MouseMotion);

    'running: loop {
        let event = event_pump.wait_event();
        match eval::commands_of_event(state, &event) {
            Ok(commands) => {
                for c in commands.iter() {
                    // note: we borrow the command here, possibly requiring some cloning when it is evaluated.
                    // todo -- we do nothing with the result; we should log it.
                    eval::command_eval(state, c)?;
                }
                let elms = eval::render_elms(&mut canvas, state)?;
                // todo actually render them
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

    let mut state = eval::load_state();

    info!("Evaluating CLI command: {:?} ...", &cliopt.command);

    match cliopt.command {
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
        CliCommand::Start => {
            do_event_loop(&mut state).unwrap();
            eval::save_state(&state);
        }
        CliCommand::Resume => {
            do_event_loop(&mut state).unwrap();
            eval::save_state(&state);
        }
        CliCommand::Replay => unimplemented!(),
        CliCommand::History => unimplemented!(),
    }
}
