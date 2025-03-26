mod args;
mod file;
mod digest;
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
        .with_level(log_level)
        .init()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;
    debug!("{:?}", CLI_OPTS);

    let (num_processed, num_duplicates) = if let Some(remote_list) = CLI_OPTS.input_file.as_ref() {
        debug!(
            "Starting digest mode dedup at {} using remote list {}",
            CLI_OPTS.local_path.display(),
            remote_list.display()
        );
        match digest::digest_mode(
            CLI_OPTS.local_path.to_path_buf(),
            remote_list.to_path_buf(),
            &CLI_OPTS.digest,
            CLI_OPTS.commit,
        )
        .await
        {
            Ok(ok) => ok,
            Err(e) => {
                error!(
                    "Digest mode dedup failed at {} using remote list {}. Error: {e}",
                    CLI_OPTS.local_path.display(),
                    remote_list.display()
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
            "Starting analysis at {}, using digest algorithm {} and writing out to {}",
            canonicalize(&CLI_OPTS.local_path).unwrap().display(),
            CLI_OPTS.digest,
            CLI_OPTS.output_file.display()
        );

        digest::analyze_path(&CLI_OPTS.local_path, &CLI_OPTS.digest, &CLI_OPTS.output_file).await?;
        return Ok(());
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
