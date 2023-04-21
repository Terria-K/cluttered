mod atlas;
mod error;
use std::path::PathBuf;

use atlas::{Config, ImageOptions, Features, TemplatePath};

use thiserror::Error;
use clap::{Command, Arg, ArgMatches};

extern crate image;

#[derive(Error, Debug)]
enum CommandError {
    #[error("There is no available command for that")]
    CommandNotFound,
    #[error("Missing one argument, please use --help")]
    MissingOneArgument,
    #[error("Unsupported format. Supported Format: .ron, .json, .toml")]
    UnsupportedFormat
}

fn main() -> anyhow::Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("config", sub_matches)) => {
            let input_path = get_path("input", sub_matches)?;
            let extension = input_path.extension();
            if let Some(extension) = extension {
                let extension = extension.to_str().unwrap_or_default();
                match extension {
                    "ron" => {
                        let config = Config::from_ron(input_path)?;
                        atlas::pack(config)?;
                    }
                    "json" => {
                        let config = Config::from_json(input_path)?;
                        atlas::pack(config)?;
                    }
                    "toml" => {
                        let config = Config::from_toml(input_path)?;
                        atlas::pack(config)?;
                    }
                    _ => Err(CommandError::UnsupportedFormat)?
                }
            }
        },
        Some(("pack", sub_matches)) => {
            if let Some(paths) = sub_matches.get_many::<PathBuf>("input") {
                let folders: Vec<PathBuf> = paths.map(|x| {
                    x.to_owned()
                }).collect();
                let output_path = get_path("output", sub_matches)?;
                let name = if let Some(name) = sub_matches.get_one::<String>("name") {
                    name.to_owned()
                } else { 
                    output_path.to_str().unwrap_or("texture-name").to_string()
                };
                let output_type = sub_matches
                    .get_one::<atlas::OutputType>("type")
                    .unwrap_or(&atlas::OutputType::Json)
                    .to_owned();
                let template_path = sub_matches
                    .get_one::<PathBuf>("template_path")
                    .map(|x| x.to_owned())
                    .map(TemplatePath::Single);

                let config = Config {
                    name,
                    output_path,
                    output_type,
                    allow_normal_output: true,
                    template_path,
                    folders,
                    image_options: ImageOptions::default(),
                    features: Features::default(),
                };
                atlas::pack(config)?;
            }

        }
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
            Command::new("config")
                .about("Specify a configuration path.")
                .arg(Arg::new("input")
                     .short('i')
                     .value_parser(clap::value_parser!(PathBuf))
                     .long("input")
                     .required(true)
                     .num_args(1)
                     .help("Specify an input for a configuration path to start packing."))
       )
        .subcommand(
            Command::new("pack")
                .about("Manually packed an image with input and output option.")
                .arg(Arg::new("input")
                     .short('i')
                     .value_parser(clap::value_parser!(PathBuf))
                     .long("input")
                     .required(true)
                     .num_args(1..)
                     .help("Specify many folders path with an images inside."))
                .arg(Arg::new("output")
                     .short('o')
                     .value_parser(clap::value_parser!(PathBuf))
                     .long("output")
                     .required(true)
                     .num_args(1)
                     .help("Specify an output folder path to output a sheet image."))
                .arg(Arg::new("type")
                     .short('t')
                     .value_parser(clap::value_parser!(atlas::OutputType))
                     .long("type")
                     .required(false)
                     .num_args(1)
                     .help("Specify an output type."))
                .arg(Arg::new("templatepath")
                     .short('a')
                     .value_parser(clap::value_parser!(PathBuf))
                     .long("template_path")
                     .required(false)
                     .num_args(1)
                     .help("Specify a Template path for template."))
                .arg(Arg::new("name")
                     .short('n')
                     .value_parser(clap::value_parser!(String))
                     .long("name")
                     .required(false)
                     .num_args(1)
                     .help("Specify an output name."))
        )
}
