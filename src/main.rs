use std::path::PathBuf;

mod hash;
mod args;
use args::CLI_OPTS;

pub enum RemoteSource<'a> {
    RemoteList(&'a PathBuf),
    RemotePath(&'a PathBuf),
}

fn main() {
    if CLI_OPTS.debug {
        println!("{:?}", CLI_OPTS);
    }

    let source = match (&CLI_OPTS.remote_list, &CLI_OPTS.remote_path) {
        (Some(_), Some(_)) => panic!("StructOpt option parsing should have prevented this"),
        (Some(list), None) => RemoteSource::RemoteList(list),
        (None, Some(path)) => RemoteSource::RemotePath(path),
        (None, None) => panic!("Must specify either -r or -R"),
    };
    
    if let RemoteSource::RemoteList(list) = source {
        hash::hash_mode(list);
        return;
    }
}
