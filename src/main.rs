mod args;
mod file;
mod hash;
mod size;
use anyhow::Result;
use args::CLI_OPTS;
use log::{debug, error, info};

fn init_logging() -> Result<()> {
    let log_level = match CLI_OPTS.verbosity {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
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

    let (num_processed, num_duplicates) = if let Some(remote_list) = CLI_OPTS.remote_list.as_ref() {
        debug!(
            "Starting hash mode dedup at {} using remote list {}",
            CLI_OPTS.local_path.display(),
            remote_list.display()
        );
        match hash::hash_mode().await {
            Ok(ok) => ok,
            Err(e) => {
                error!(
                    "Hash mode dedup failed at {} using remote list {}. Error: {e}",
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
        match size::size_mode().await {
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
        anyhow::bail!("We're on event horizon? Impossible! Just like this error")
    };

    info!(
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
