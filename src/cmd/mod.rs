use elements::AddressParams;
use serde::{Deserialize, Serialize};

pub mod call;
pub mod config;
pub use self::call::{OptAssetParams, OptionContract, BTC_ASSET, CTRL_PK, CTRL_SK};
pub use self::config::Config;

/// Known Elements networks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    ElementsRegtest,
    Liquid,
}

impl Network {
    pub fn from_params(params: &'static AddressParams) -> Option<Network> {
        match params {
            &AddressParams::ELEMENTS => Some(Network::ElementsRegtest),
            &AddressParams::LIQUID => Some(Network::Liquid),
            _ => None,
        }
    }

    pub fn address_params(self) -> &'static AddressParams {
        match self {
            Network::ElementsRegtest => &AddressParams::ELEMENTS,
            Network::Liquid => &AddressParams::LIQUID,
        }
    }
}
/// Build a list of all built-in subcommands.
pub fn subcommands<'a>() -> Vec<clap::App<'a, 'a>> {
    vec![call::subcommand(), config::cmd_create()]
}

/// Construct a new command option.
pub fn opt<'a>(name: &'static str, help: &'static str) -> clap::Arg<'a, 'a> {
    clap::Arg::with_name(name).long(name).help(help)
}

/// Construct a new positional argument.
pub fn arg<'a>(name: &'static str, help: &'static str) -> clap::Arg<'a, 'a> {
    clap::Arg::with_name(name).help(help).takes_value(true)
}

/// Create a new subcommand group using the template that sets all the common settings.
/// This is not intended for actual commands, but for subcommands that host a bunch of other
/// subcommands.
pub fn subcommand_group<'a>(name: &'static str, about: &'static str) -> clap::App<'a, 'a> {
    clap::SubCommand::with_name(name).about(about).settings(&[
        clap::AppSettings::SubcommandRequiredElseHelp,
        clap::AppSettings::DisableHelpSubcommand,
        clap::AppSettings::VersionlessSubcommands,
        clap::AppSettings::UnifiedHelpMessage,
    ])
}

/// Create a new subcommand using the template that sets all the common settings.
pub fn subcommand<'a>(name: &'static str, about: &'static str) -> clap::App<'a, 'a> {
    clap::SubCommand::with_name(name)
        .about(about)
        .setting(clap::AppSettings::DisableHelpSubcommand)
}

pub fn opts_networks<'a>() -> Vec<clap::Arg<'a, 'a>> {
    vec![
        clap::Arg::with_name("elementsregtest")
            .long("elementsregtest")
            .short("r")
            .help("run in elementsregtest mode")
            .takes_value(false)
            .required(false),
        clap::Arg::with_name("liquid")
            .long("liquid")
            .help("run in liquid mode")
            .takes_value(false)
            .required(false),
    ]
}

pub fn network<'a>(matches: &clap::ArgMatches<'a>) -> Network {
    if matches.is_present("elementsregtest") {
        Network::ElementsRegtest
    } else if matches.is_present("liquid") {
        Network::Liquid
    } else {
        Network::ElementsRegtest
    }
}

pub fn opt_yaml<'a>() -> clap::Arg<'a, 'a> {
    clap::Arg::with_name("yaml")
        .long("yaml")
        .short("y")
        .help("print output in YAML instead of JSON")
        .takes_value(false)
        .required(false)
}

pub fn print_output<'a, T: serde::Serialize>(matches: &clap::ArgMatches<'a>, out: &T) {
    if matches.is_present("yaml") {
        serde_yaml::to_writer(::std::io::stdout(), &out).unwrap();
    } else {
        serde_json::to_writer_pretty(::std::io::stdout(), &out).unwrap();
    }
}
