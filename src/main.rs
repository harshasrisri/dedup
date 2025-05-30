mod args;
mod analyze;
mod remote;
mod local;
mod inplace;
mod digest;
mod file;
mod size;
use std::fs::canonicalize;

use anyhow::Result;
use args::{DedupOpts, OperatingMode};
use clap::Parser;
use log::{debug, error};

fn init_logging(verbosity: u8) -> Result<()> {
    let log_level = match verbosity {
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
    let cli_args = DedupOpts::parse();
    let (mut num_processed, mut num_duplicates) = (0, 0);

    init_logging(cli_args.verbosity)?;

    match cli_args.mode {
        OperatingMode::Analyze(args) => {
            debug!(
                "Starting digest mode analysis at {}, using {} and writing out to {}",
                canonicalize(&args.local_path).unwrap().display(),
                args.digest,
                args.output_file.display()
            );

            match digest::analyze_path(
                &args.local_path,
                &args.digest,
                &args.output_file,
            )
            .await
            {
                Ok(()) => return Ok(()),
                Err(e) => {
                    error!(
                        "Digest mode analysis failed at {} using {} and writing out to {}. Error: {e}",
                        canonicalize(args.local_path).unwrap().display(),
                        args.digest,
                        args.output_file.display()
                    );
                    std::process::exit(1);
                }
            };
        },

        OperatingMode::Remote(args) => {
            debug!(
                "Starting digest mode dedup at {} using input file {}",
                args.local_path.display(),
                args.input_file.as_ref().unwrap().display()
            );
            (num_processed, num_duplicates) = match digest::digest_mode(
                args.local_path.to_path_buf(),
                args.input_file.as_ref().unwrap().to_path_buf(),
                &args.digest,
                cli_args.commit,
            )
            .await
            {
                Ok(ok) => ok,
                Err(e) => {
                    error!(
                        "Digest mode dedup failed at {} using input file {}. Error: {e}",
                        args.local_path.display(),
                        args.input_file.unwrap().display()
                    );
                    std::process::exit(1);
                }
            }

        },
        OperatingMode::Local(args) => {
            debug!(
                "Starting size mode dedup as {} using remote path {}",
                &args.local_path.display(),
                args.reference_path.as_ref().unwrap().display()
            );
            (num_processed, num_duplicates) = match size::size_mode(
                args.local_path.to_path_buf(),
                args.reference_path.as_ref().unwrap().to_path_buf(),
                cli_args.commit,
            )
            .await
            {
                Ok(ok) => ok,
                Err(e) => {
                    error!(
                        "Size mode dedup failed at {} using remote path {}. Error: {e}",
                        &args.local_path.display(),
                        args.reference_path.unwrap().display()
                    );
                    std::process::exit(1);
                }
            }
        },
        OperatingMode::InPlace(_args) => {},
    };

    println!(
        "{} files processed. {} Duplicates {}",
        num_processed,
        num_duplicates,
        if cli_args.commit {
            "deleted".to_string()
        } else {
            "found".to_string()
        }
    );

    Ok(())
}
