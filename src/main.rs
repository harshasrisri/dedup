use anyhow::Result;
use clap::{Parser, Subcommand};
use dedup::analyze::Analyze;
use dedup::inplace::InPlace;
use dedup::local::Local;
use dedup::remote::Remote;
use dedup::size;
use log::{debug, error};
use std::fs::canonicalize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct DedupOpts {
    /// Performs a dry run by default. Use this option to commit file deletions
    #[arg(short, long)]
    pub commit: bool,

    /// Flag count for log verbosity (info(1), debug(2), trace(3)) [default: warn(0)]
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub mode: OperatingMode,
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum OperatingMode {
    Analyze(Analyze),
    Remote(Remote),
    Local(Local),
    InPlace(InPlace),
}
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
            match args.analyze().await {
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
        }

        OperatingMode::Remote(args) => {
            (num_processed, num_duplicates) = match args.dedup(cli_args.commit).await
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
        }

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
        }

        OperatingMode::InPlace(_args) => {}
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
