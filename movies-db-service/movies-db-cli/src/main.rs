mod options;

use anyhow::Result;
use log::{error, info};
use movies_db::{Options as ServiceOptions, Service, SimpleMoviesIndex};
use options::Options;

use clap::Parser;

use log::LevelFilter;

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
async fn run_program() -> Result<()> {
    let options = parse_args()?;
    initialize_logging(LevelFilter::from(options.log_level));

    let service_options = ServiceOptions {
        root_dir: options.root_dir,
    };

    let service: Service<SimpleMoviesIndex> = Service::new(&service_options)?;
    service.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    match run_program().await {
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
