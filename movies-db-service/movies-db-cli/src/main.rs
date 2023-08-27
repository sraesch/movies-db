mod options;

use anyhow::Result;
use log::{error, info};
use movies_db::{
    file_storage::FileStorage, Options as ServiceOptions, Service,
    SqliteMoviesIndex as MoviesIndexImpl,
};
use options::Options;

use clap::Parser;

use log::LevelFilter;

use std::io::Write;

/// Parses the program arguments and returns None, if no arguments were provided and Some otherwise.
fn parse_args() -> Result<Options> {
    let options = Options::parse();
    Ok(options)
}

/// Initializes the program logging
fn initialize_logging(filter: LevelFilter) {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(filter)
        .init();
}

/// Runs the program.
async fn run_program() -> Result<()> {
    let options = parse_args()?;
    initialize_logging(LevelFilter::from(options.log_level));

    let service_options: ServiceOptions = options.into();

    let service: Service<MoviesIndexImpl, FileStorage> = Service::new(&service_options)?;
    service.run().await?;

    Ok(())
}

#[actix_web::main]
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
