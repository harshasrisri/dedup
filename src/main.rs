mod args;
mod digest;
mod file;
mod size;
use std::fs::canonicalize;

use anyhow::Result;
use args::CLI_OPTS;
use log::{debug, error};

fn init_logging() -> Result<()> {
    let log_level = match CLI_OPTS.verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(log_level)
        .without_timestamps()
        .init()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    let (num_processed, num_duplicates) = if let Some(input_file) = CLI_OPTS.input_file.as_ref() {
        debug!(
            "Starting digest mode dedup at {} using input file {}",
            CLI_OPTS.local_path.display(),
            input_file.display()
        );
        match digest::digest_mode(
            CLI_OPTS.local_path.to_path_buf(),
            input_file.to_path_buf(),
            &CLI_OPTS.digest,
            CLI_OPTS.commit,
        )
        .await
        {
            Ok(ok) => ok,
            Err(e) => {
                error!(
                    "Digest mode dedup failed at {} using input file {}. Error: {e}",
                    CLI_OPTS.local_path.display(),
                    input_file.display()
                );
                std::process::exit(1);
            }
        }
    } else if let Some(remote_path) = CLI_OPTS.remote_path.as_ref() {
        debug!(
            "Starting size mode dedup as {} using remote path {}",
            &CLI_OPTS.local_path.display(),
            remote_path.display()
        );
        match size::size_mode(
            CLI_OPTS.local_path.to_path_buf(),
            remote_path.to_path_buf(),
            CLI_OPTS.commit,
        )
        .await
        {
            Ok(ok) => ok,
            Err(e) => {
                error!(
                    "Size mode dedup failed at {} using remote path {}. Error: {e}",
                    &CLI_OPTS.local_path.display(),
                    remote_path.display()
                );
                std::process::exit(1);
            }
        }
    } else {
        debug!(
            "Starting digest mode analysis at {}, using {} and writing out to {}",
            canonicalize(&CLI_OPTS.local_path).unwrap().display(),
            CLI_OPTS.digest,
            CLI_OPTS.output_file.display()
        );

        match digest::analyze_path(
            &CLI_OPTS.local_path,
            &CLI_OPTS.digest,
            &CLI_OPTS.output_file,
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(e) => {
                error!(
                    "Digest mode analysis failed at {} using {} and writing out to {}. Error: {e}",
                    canonicalize(&CLI_OPTS.local_path).unwrap().display(),
                    CLI_OPTS.digest,
                    CLI_OPTS.output_file.display()
                );
                std::process::exit(1);
            }
        }
    };

    println!(
        "{} files processed. {} Duplicates {}",
        num_processed,
        num_duplicates,
        if CLI_OPTS.commit {
            "deleted".to_string()
        } else {
            "found".to_string()
        }
    );

    Ok(())
}
