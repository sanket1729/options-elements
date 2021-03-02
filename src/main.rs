extern crate bitcoin;
extern crate elements;
extern crate elements_miniscript as miniscript;

/// Code from hal project by Steven Roose
extern crate log;
extern crate base64;
extern crate chrono;
extern crate clap;
extern crate fern;
extern crate hex;
extern crate jobserver;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate shell_escape;

use std::env;
use std::io::{self, Write};
use std::panic;
use std::process;

pub mod cmd;
mod process_builder;
pub mod util;

/// Setup logging with the given log level.
fn setup_logger(lvl: log::LevelFilter) {
    fern::Dispatch::new()
        .format(|out, message, _record| out.finish(format_args!("{}", message)))
        .level(lvl)
        .chain(std::io::stderr())
        .apply()
        .expect("error setting up logger");
}

/// Create the main app object.
fn init_app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("el-opt")
        .version(clap::crate_version!())
        .author("Sanket K <sanket1729@gmail.com>")
        .about("Options on liquid - Call/Put options on liquid")
        .settings(&[
            clap::AppSettings::GlobalVersion,
            clap::AppSettings::UnifiedHelpMessage,
            clap::AppSettings::VersionlessSubcommands,
            clap::AppSettings::AllowExternalSubcommands,
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::AllArgsOverrideSelf,
        ])
        .subcommands(cmd::subcommands())
        .arg(
            cmd::opt("verbose", "Print verbose logging output to stderr")
                .short("v")
                .takes_value(false)
                .global(true),
        )
}


fn main() {
    // Apply a custom panic hook to print a more user-friendly message
    // in case the execution fails.
    // We skip this for people that are interested in the panic message.
    if env::var("RUST_BACKTRACE").unwrap_or(String::new()) != "1" {
        panic::set_hook(Box::new(|info| {
            let message = if let Some(m) = info.payload().downcast_ref::<String>() {
                m
            } else if let Some(m) = info.payload().downcast_ref::<&str>() {
                m
            } else {
                "No error message provided"
            };
            eprintln!("Execution failed: {}", message);
            process::exit(1);
        }));
    }

    let app = init_app();
    let matches = app.clone().get_matches();

    // Enable logging in verbose mode.
    match matches.is_present("verbose") {
        true => setup_logger(log::LevelFilter::Trace),
        false => setup_logger(log::LevelFilter::Warn),
    }

    match matches.subcommand() {
        ("", _) => {
            app.write_help(&mut io::stderr()).unwrap();
            io::stderr().write(b"\n").unwrap();
            process::exit(1);
        }
        ("call", Some(ref m)) => cmd::call::execute(&m),
        ("init", Some(ref m)) => cmd::config::exec_init(&m),
        (cmd, _subcommand_args) => {
            // Try execute an external subcommand.
            panic!("no such subcommand: `{}`", cmd);
        }
    }
}
