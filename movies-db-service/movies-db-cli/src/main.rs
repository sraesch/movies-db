mod options;

use anyhow::Result;
use log::{error, info};
use options::Options;

use clap::{Parser};

use log::{LevelFilter};

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
fn initialize_logging(filter: LevelFilter) {
    simple_logging::log_to(std::io::stdout(), filter);
}


/// Runs the program.
fn run_program() -> Result<()> {
    let options = parse_args()?;
    initialize_logging(LevelFilter::from(options.log_level));

    Ok(())
}

fn main() {
    match run_program() {
        Ok(()) => {
            info!("SUCCESS");
        }
        Err(err) => {
            error!("Error: {}", err);
            error!("FAILED");

            std::process::exit(-1);
        }
    }
}