#![feature(nll)] // Rust non-lexical lifetimes

// SDL: Keyboard/mouse input events, multi-media output abstractions:
extern crate sdl2;

// Logging:
#[macro_use] extern crate log;
extern crate env_logger;

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

fn init_log(verbose:bool) {
    use log::LevelFilter;
    use env_logger::{Builder, WriteStyle};
    let mut builder = Builder::new();
    builder.filter(None, if verbose { LevelFilter::Trace } else { LevelFilter::Debug })
        .write_style(WriteStyle::Always)
        .init();
}

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

    #[structopt(name  = "sdl-test",
                about = "Test SDL library.")]
    SdlTest,

    #[structopt(name  = "goto-place",
                about = "Go to an existing, previously-named place.")]
    GotoPlace{
        name: String
    },

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
        CliCommand::SdlTest => Command::SdlTest,
        CliCommand::MakeTime{name:n} => Command::MakeTime(translate_time_name(&n)),
        CliCommand::MakePlace{name:n} => Command::MakePlace(translate_place_name(&n)),
        CliCommand::GotoPlace{name:n} => Command::GotoPlace(n.to_string()),
        CliCommand::BitmapMake8x8 => Command::Bitmap(bitmap::Command::Init(bitmap::InitCommand::Make8x8)),
        CliCommand::BitmapToggle => Command::Bitmap(bitmap::Command::Edit(bitmap::EditCommand::Toggle)),
        CliCommand::BitmapMove{dir:ref d} => {
            let dir = match d {
                &CliDir2D::Up    => Dir2D::Up,
                &CliDir2D::Down  => Dir2D::Down,
                &CliDir2D::Left  => Dir2D::Left,
                &CliDir2D::Right => Dir2D::Right,
            };
            Command::Bitmap(bitmap::Command::Edit(bitmap::EditCommand::Move(dir)))
        }
        //_ => unimplemented!("translate_command({:?})", clicmd),
    }
}


pub fn sdl_event_loop() -> Result<(), String> {
    use sdl2::rect::{Rect};
    use sdl2::pixels::Color;
    use sdl2::event::Event;
    //use sdl2::mouse::MouseButton;
    use sdl2::keyboard::Keycode;
    //use sdl2::video::{Window, WindowContext};
    //use sdl2::render::{Canvas, Texture, TextureCreator};

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // the window is the representation of a window in your operating system,
    // however you can only manipulate properties of that window, like its size, whether it's
    // fullscreen, ... but you cannot change its content without using a Canvas or using the
    // `surface()` method.
    let zoom = 32u32;
    let width = 8u32;
    let height = 8u32;
    let window = video_subsystem
        .window("zoom-quilt-maker",
                8 * zoom + 1,
                8 * zoom + 1)
        .position_centered()
        //.fullscreen()
        //.fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    // the canvas allows us to both manipulate the property of the window and to change its content
    // via hardware or software rendering. See CanvasBuilder for more info.
    let mut canvas = window.into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);

    // grid border is a single background rect:
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.fill_rect(
        Rect::new(
            0,
            0,
            width * zoom + 1,
            height * zoom + 1,
        )
    )?;

    // grid cells are rects:
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    for x in 0i32..width as i32 {
        for y in 0i32..height as i32 {
            if y & 5 == 0 && x % 3 == 0 ||
                x % 2 == 0 && y % 3 == 0
            {
                canvas.draw_rect(
                    Rect::new(
                        x * zoom as i32 + 2,
                        y * zoom as i32 + 2,
                        zoom            - 4,
                        zoom            - 4,
                    )
                )?;
            } else {
                canvas.fill_rect(
                    Rect::new(
                        x * zoom as i32 + 2,
                        y * zoom as i32 + 2,
                        zoom            - 4,
                        zoom            - 4,
                    )
                )?;
            }
        }
    }
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape), ..
                } => {
                    break 'running
                },
                Event::KeyDown {
                    keycode: kc,
                    repeat: false, ..
                } => {
                    debug!("KeyDown: {:?}", kc)
                },
                Event::MouseButtonDown {
                    x, y,
                    mouse_btn: b,
                    ..
                } => {
                    debug!("MouseButtonDown: {:?} {:?} {:?}", x, y, b)
                },
                _ => {}
            }
        }
    }

    Ok(())
}


fn main() {
    let cliopt  = CliOpt::from_args();
    init_log(cliopt.verbose);

    let mut state = eval::init();

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
        CliCommand::SdlTest => {
            sdl_event_loop().unwrap();
        }
        CliCommand::MakeTime{..}    => { eval::eval(&mut state, &command) }
        CliCommand::MakePlace{..}   => { eval::eval(&mut state, &command) }
        CliCommand::GotoPlace{..}   => { eval::eval(&mut state, &command) }
        CliCommand::BitmapMake8x8   => { eval::eval(&mut state, &command) }
        CliCommand::BitmapToggle    => { eval::eval(&mut state, &command) }
        CliCommand::BitmapMove{..}  => { eval::eval(&mut state, &command) }
        CliCommand::ReadLine        => {
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
}
