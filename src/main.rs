mod args;
mod file;
mod hash;
mod size;
use args::CLI_OPTS;

fn main() {
    if CLI_OPTS.debug {
        println!("{:?}", CLI_OPTS);
    }

    if let Some(remote_list) = &CLI_OPTS.remote_list {
        hash::hash_mode(remote_list);
        return;
    }

    let remote_path = CLI_OPTS.remote_path.as_ref().expect("Expected a remote path CLI option");

    if std::fs::canonicalize(remote_path).unwrap()
        == std::fs::canonicalize(&CLI_OPTS.local_path).unwrap()
    {
        eprintln!(
            "In-place deduplication not yet supported. {} and {} are the same path.",
            remote_path.display(),
            CLI_OPTS.local_path.display()
        );
        return;
    }
}
