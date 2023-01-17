mod atlas;
use std::path::PathBuf;

use atlas::PackerConfig;

use thiserror::Error;
use clap::{Command, Arg, ArgMatches};

extern crate image;

#[derive(Error, Debug)]
enum CommandError {
    #[error("There is no available command for that")]
    CommandNotFound,
    #[error("Missing one argument, please use --help")]
    MissingOneArgument
}

fn main() -> anyhow::Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("json-path", sub_matches)) => {
            let input_path = get_path("input", sub_matches)?;
            let config = PackerConfig::from_json(input_path)?;
            atlas::pack(config)?;
        },
        Some(("ron-path", sub_matches)) => {
            let input_path = get_path("input", sub_matches)?;
            let config = PackerConfig::from_ron(input_path)?;
            atlas::pack(config)?;
        },
        _ => Err(CommandError::CommandNotFound)?,
    }
    Ok(())
}

#[inline]
fn get_path(id: &str, matches: &ArgMatches) -> anyhow::Result<PathBuf, CommandError> {
    match matches.get_one::<PathBuf>(id) {
        Some(path) => Ok(path.to_owned()),
        None => Err(CommandError::MissingOneArgument)
    }
}

fn cli() -> Command {
    Command::new("pack")
        .about("Pack an images")
        .subcommand_required(false)
        .subcommand(
            Command::new("json-path")
                .about("Specify a .json configuration path.")
                .arg(Arg::new("input")
                     .short('i')
                     .value_parser(clap::value_parser!(PathBuf))
                     .long("input")
                     .required(true)
                     .num_args(1)
                     .help("Specify an input for .json path to start packing."))
       )
        .subcommand(
            Command::new("ron-path")
                .about("Specify a .ron configuration path.")
                .arg(Arg::new("input")
                     .short('i')
                     .value_parser(clap::value_parser!(PathBuf))
                     .long("input")
                     .required(true)
                     .num_args(1)
                     .help("Specify an input for .ron path to start packing."))
       )
}
