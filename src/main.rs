mod args;
mod file;
mod modes;
use anyhow::Result;
use args::CLI_OPTS;

fn main() -> Result<()> {
    if CLI_OPTS.debug {
        println!("{:?}", CLI_OPTS);
    }

    if let Some(remote_list) = &CLI_OPTS.remote_list {
        return modes::hash_mode(remote_list);
    }

    let remote_path = CLI_OPTS
        .remote_path
        .as_ref()
        .expect("Expected a remote path CLI option");

    if std::fs::canonicalize(remote_path).unwrap()
        == std::fs::canonicalize(&CLI_OPTS.local_path).unwrap()
    {
        anyhow::bail!(
            "In-place deduplication not yet supported. {} and {} are the same path.",
            remote_path.display(),
            CLI_OPTS.local_path.display()
        );
    }

    modes::size_mode()
}
