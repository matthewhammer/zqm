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
use zoom_quilt_maker::{
    eval, bitmap,
    Name, Command, Dir2D
};

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
    #[structopt(name  = "make-time",
                about = "Give a meaningful name for a fresh time.")]
    #[structopt(raw(setting="clap::AppSettings::DeriveDisplayOrder"))]
    MakeTime{
        #[structopt(subcommand)]
        name: CliTimeName
    },

    #[structopt(name  = "make-place",
                about = "Give a meaningful name for a fresh place.")]
    #[structopt(raw(setting="clap::AppSettings::DeriveDisplayOrder"))]
    MakePlace{
        #[structopt(subcommand)]
        name: CliPlaceName
    },

    #[structopt(name  = "read-line",
                about = "Read a line of text from stdin; archive it.")]
    ReadLine,

    #[structopt(name  = "goto-place",
                about = "Go to an existing, previously-named place.")]
    GotoPlace{
        name: String
    },

    #[structopt(name  = "bitmap-editor",
                about = "Resume editing a bitmap.")]
    BitmapEditor,

    #[structopt(name  = "bitmap-make-8x8",
                about = "Make a new monochrome 8x8 bitmap.")]
    BitmapMake8x8,

    #[structopt(name  = "bitmap-move",
                about = "Move bitmap edit cursor.")]
    #[structopt(raw(setting="clap::AppSettings::DeriveDisplayOrder"))]
    BitmapMove{
        #[structopt(subcommand)]
        dir:CliDir2D
    },

    #[structopt(name  = "bitmap-toggle",
                about = "Toggle bit at bitmap edit cursor.")]
    BitmapToggle,

    /////////////////////////////////////////////////////////////////////////

    #[structopt(name  = "version",
                about = "Display version.")]
    Version,

    #[structopt(name  = "completions",
                about = "Generate shell scripts for zqm subcommand auto-completions.")]
    Completions{
        shell: Shell,
    },
}

#[derive(StructOpt, Debug)]
enum CliDir2D {
    #[structopt(name = "up", about = "↑ Move up")]
    Up,
    #[structopt(name = "down", about = "↓ Move down")]
    Down,
    #[structopt(name = "left", about = "← Move left")]
    Left,
    #[structopt(name = "right", about = "→ Move right")]
    Right
}

#[derive(StructOpt, Debug)]
enum CliTimeName {
    #[structopt(name  = "from-date",
                about = "Determine the time name from the current date and time.")]
    FromDate,

    #[structopt(name  = "by-name",
                about = "Give the time name explicitly as a string argument.")]
    ByName{
        name: String,
    }
}

#[derive(StructOpt, Debug)]
enum CliPlaceName {
    #[structopt(name  = "from-wd",
                about = "Determine the place name from the current working directory.")]
    FromWd,

    #[structopt(name  = "by-name",
                about = "Give the place name explicitly as a string argument.")]
    ByName{
        name: String,
    }
}

fn translate_time_name(cli_time_name:&CliTimeName) -> Name {
    match cli_time_name.clone() {
        CliTimeName::ByName{name:n} => n.to_string(),
        CliTimeName::FromDate => {
            unimplemented!()
        }
    }
}

fn translate_place_name(cli_place_name:&CliPlaceName) -> Name {
    match cli_place_name.clone() {
        CliPlaceName::ByName{name:n} => n.to_string(),
        CliPlaceName::FromWd => {
            unimplemented!()
        }
    }
}

// translate CLI commands into the forms that we archive
fn translate_command(clicmd:&CliCommand) -> Command {
    match clicmd.clone() {
        CliCommand::Version => Command::Version,
        CliCommand::Completions{shell:s} => Command::Completions(s.to_string()),
        CliCommand::ReadLine => Command::ReadLine,
        CliCommand::MakeTime{name:n} => Command::MakeTime(translate_time_name(&n)),
        CliCommand::MakePlace{name:n} => Command::MakePlace(translate_place_name(&n)),
        CliCommand::GotoPlace{name:n} => Command::GotoPlace(n.to_string()),
        CliCommand::BitmapEditor  => Command::Bitmap(bitmap::Command::Editor),
        CliCommand::BitmapMake8x8 => Command::Bitmap(bitmap::Command::Init(bitmap::InitCommand::Make8x8)),
        CliCommand::BitmapToggle  => Command::Bitmap(bitmap::Command::Edit(bitmap::EditCommand::Toggle)),
        CliCommand::BitmapMove{dir:ref d} => {
            let dir = match d {
                &CliDir2D::Up    => Dir2D::Up,
                &CliDir2D::Down  => Dir2D::Down,
                &CliDir2D::Left  => Dir2D::Left,
                &CliDir2D::Right => Dir2D::Right,
            };
            Command::Bitmap(bitmap::Command::Edit(bitmap::EditCommand::MoveRel(dir)))
        }
        //_ => unimplemented!("translate_command({:?})", clicmd),
    }
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


pub fn sdl2_bitmap_editor(editor: &mut bitmap::Editor) -> Result<(), String> {
    //use sdl2::event::Event;
    //use sdl2::keyboard::Keycode;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let zoom = 64u32;
    let window = video_subsystem
        .window("zoom-quilt-make::bitmap",
                8 * zoom + 1,
                8 * zoom + 1)
        .position_centered()
    //.fullscreen()
    //.fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    info!("Using SDL_Renderer \"{}\"", canvas.info().name);

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        let event = event_pump.wait_event();
        match bitmap::io::consume_input(event) {
            Ok(commands) => {
                for c in commands.iter() {
                    bitmap::semantics::editor_eval(
                        editor, &bitmap::Command::Edit(c.clone())
                    )?;
                };
                match editor.state {
                    None => (),
                    Some(ref st) =>
                        bitmap::io::produce_output(&mut canvas, st)?
                }
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

    info!("command := translate({:?})", &cliopt.command);
    let command = translate_command(&cliopt.command);
    info!("command := {:?}", &command);

    // todo(hammer): move/"lift" this into the zqm library
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
        CliCommand::BitmapEditor => {
            match eval::eval(&mut state, &command) {
                Ok(()) => {
                    sdl2_bitmap_editor(&mut state.bitmap_editor).unwrap();
                    info!("to do: bitmap edit history saved at {:?}", state.locus);
                },
                Err(err) => {
                    warn!("no existing bitmap; creating an empty one...");
                    bitmap::semantics::editor_eval(
                        &mut state.bitmap_editor,
                        &bitmap::Command::Init(
                            bitmap::InitCommand::Make8x8
                        )
                    ).unwrap();
                    sdl2_bitmap_editor(&mut state.bitmap_editor).unwrap();
                    info!("to do: bitmap edit history saved at {:?}", state.locus);
                }
            }
        },
        CliCommand::BitmapMake8x8     => { eval::eval(&mut state, &command).unwrap(); }
        CliCommand::MakeTime{..}      => { eval::eval(&mut state, &command).unwrap(); }
        CliCommand::MakePlace{..}     => { eval::eval(&mut state, &command).unwrap(); }
        CliCommand::GotoPlace{..}     => { eval::eval(&mut state, &command).unwrap(); }
        CliCommand::BitmapToggle      => { eval::eval(&mut state, &command).unwrap(); }
        CliCommand::BitmapMove{..}    => { eval::eval(&mut state, &command).unwrap(); }
        CliCommand::ReadLine          => {
            let mut input = String::new();
            debug!("reading a line from stdin to store at {:?}", state.locus);
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
                    trace!("read {} bytes:", n);
                    trace!("\"{}\"", input);
                }
                Err(e) => error!("{}", e),
            }
        }
    }
    eval::save_state(&state);
}
