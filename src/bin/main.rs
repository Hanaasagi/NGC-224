#![allow(non_snake_case)]
use std::process;

use clap::{App, Arg};
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use NGC224::gameboy::Config;
use NGC224::gameboy::Emulator;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .info(Color::BrightGreen)
        .error(Color::BrightRed)
        .warn(Color::BrightYellow)
        .debug(Color::BrightMagenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {} - {:<36}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn parse_cmd() -> Result<Config, Box<dyn std::error::Error>> {
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .required(true)
                .help("the rom path")
                .takes_value(true),
        )
        .get_matches();

    if let Some(path) = matches.value_of("path") {
        return Ok(Config::new(path.to_string()));
    }
    Err("command line parse error".into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;
    // env_logger::init();

    info!("GameBoy Start!!!");
    info!("PID is {}", process::id());
    let config = parse_cmd()?;
    // let config = Config::new("./09-op r,r.gb".to_string());

    // lazy_static!{

    //     static ref emulator: Emulator = ;

    // }
    // let emulator = Box::leak(Box::new(Emulator::new(config)));

    let mut emulator = Emulator::new(config);

    emulator.run();

    Ok(())
}
