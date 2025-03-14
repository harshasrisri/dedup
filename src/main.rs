mod args;
mod file;
mod modes;
use anyhow::Result;
use args::CLI_OPTS;
use log::debug;

fn init_logging() -> Result<()> {
    let log_level = match CLI_OPTS.verbosity {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    simple_logger::SimpleLogger::new()
        .with_level(log_level)
        .init()?;
    Ok(())
}


fn main() -> Result<()> {
    init_logging()?;
    debug!("{:?}", CLI_OPTS);

    let (num_processed, num_duplicates) = if let Some(remote_list) = &CLI_OPTS.remote_list {
        debug!(
            "Starting hash mode dedup at {} using remote list {}",
            &CLI_OPTS.local_path.display(),
            remote_list.display()
        );
        modes::hash_mode()?
    } else if let Some(remote_path) = &CLI_OPTS.remote_path {
        debug!(
            "Starting size mode dedup as {} using remote path {}",
            &CLI_OPTS.local_path.display(),
            remote_path.display()
        );
        modes::size_mode()?
    } else {
        anyhow::bail!("We're on event horizon? Impossible! Just like this error")
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
