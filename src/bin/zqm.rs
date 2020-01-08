// SDL: Keyboard/mouse input events, multi-media output abstractions:
extern crate sdl2;

// Logging:
#[macro_use] extern crate log;
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
#[structopt(name="zqm",raw(setting="clap::AppSettings::DeriveDisplayOrder"))]
struct CliOpt {
    /// Enable debug logging
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    #[structopt(subcommand)]
    command: CliCommand,
}

#[derive(StructOpt, Debug)]
enum CliCommand {
    #[structopt(name  = "start",
                about = "Start interactively.")]
    Start,

    #[structopt(name  = "version",
                about = "Display version.")]
    Version,

    #[structopt(name  = "completions",
                about = "Generate shell scripts for zqm subcommand auto-completions.")]
    Completions{
        shell: Shell,
    },
}

fn init_log(verbose:bool) {
    use log::LevelFilter;
    use env_logger::{Builder, WriteStyle};
    let mut builder = Builder::new();
    builder.filter(None,
                   if verbose {
                       LevelFilter::Trace
                   }
                   else {
                       LevelFilter::Debug
                   })
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
        .window("zoom-quilt-maker",
                grid_size.0 * zoom + 1,
                grid_size.1 * zoom + 1)
        .position_centered()
        .resizable()
        //.input_grabbed()
        .fullscreen()
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas()
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
        match eval::consume_input(state, event) {
            Ok(commands) => {
                for c in commands.iter() {
                    eval::eval_command(state, c)?;
                }
                let elms = eval::render_elms(&mut canvas, state)?;
                // todo actually render them
                drop(elms);
            }
            Err(()) => {
                break 'running
            }
        }
    };
    Ok(())
}

fn main() {
    let cliopt  = CliOpt::from_args();
    init_log(cliopt.verbose);

    let mut state = eval::load_state();

    info!("CLI command {:?}", &cliopt.command);

    match cliopt.command {
        CliCommand::Version => {
            const VERSION: &'static str = env!("CARGO_PKG_VERSION");
            println!("{}", VERSION);
        }
        CliCommand::Completions{shell:s} => {
            // see also: https://clap.rs/effortless-auto-completion/
            //
            CliOpt::clap().gen_completions_to("zqm", s, &mut io::stdout());
            info!("done")
        }
        CliCommand::Start => {
            do_event_loop(&mut state).unwrap();
            eval::save_state(&state);
        }
    }
}
