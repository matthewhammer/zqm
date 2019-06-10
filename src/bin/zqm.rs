#![feature(nll)] // Rust non-lexical lifetimes

// Logging:
#[macro_use] extern crate log;
extern crate env_logger;

// CLI: Representation and processing:
extern crate clap;
//use clap::Shell;

extern crate structopt;
use structopt::StructOpt;

//use std::io;

// Unix OS:
//use std::process::Command as UnixCommand;

// ZQM:
extern crate zoom_quilt_maker;
use zoom_quilt_maker::{
    eval, bitmap,
    Command, Dir2D
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
    MakeTime{
        name: String
    },

    #[structopt(name  = "make-place",
                about = "Give a meaningful name for a fresh place.")]
    MakePlace{
        name: String
    },

    #[structopt(name  = "read-line",
                about = "Read a line of text from stdin; archive it.")]
    ReadLine,

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

    #[structopt(name  = "shell-completions",
                about = "Generate shell scripts for zqm subcommand auto-completions.")]
    ShellCompletions{
        shell: String,
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

// translate CLI commands into the forms that we archive
fn translate_command(clicmd:&CliCommand) -> Command {
    match clicmd.clone() {
        CliCommand::Version   => Command::Version,
        CliCommand::ShellCompletions{shell:s} => Command::ShellCompletions(s.to_string()),
        CliCommand::ReadLine  => Command::ReadLine,
        CliCommand::MakeTime{name:n}  => Command::MakeTime(n.to_string()),
        CliCommand::MakePlace{name:n} => Command::MakePlace(n.to_string()),
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
        CliCommand::ShellCompletions{shell:_s} => {
            // see also: https://clap.rs/effortless-auto-completion/
            //
            //CliOpt::build_cli().gen_completions_to("zqm", s, &mut io::stdout())
            unimplemented!()
        }
        CliCommand::MakeTime{..}    => { eval::eval(&mut state, &command) }
        CliCommand::MakePlace{..}   => { eval::eval(&mut state, &command) }
        CliCommand::GotoPlace{..}   => { eval::eval(&mut state, &command) }
        CliCommand::BitmapMake8x8   => { eval::eval(&mut state, &command) }
        CliCommand::BitmapToggle    => { eval::eval(&mut state, &command) }
        CliCommand::BitmapMove{..}  => { eval::eval(&mut state, &command) }
        CliCommand::ReadLine        => {
            use std::io;
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
