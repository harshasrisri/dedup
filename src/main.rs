mod args;
mod hash;
use args::CLI_OPTS;

fn main() {
    if CLI_OPTS.debug {
        println!("{:?}", CLI_OPTS);
    }

    let _remote_path;
    match (&CLI_OPTS.remote_list, &CLI_OPTS.remote_path) {
        (Some(_), Some(_)) => panic!("StructOpt option parsing should have prevented this"),
        (Some(list), None) => {
            hash::hash_mode(list);
            return;
        }
        (None, Some(path)) => _remote_path = path,
        (None, None) => panic!("Must specify either -r or -R"),
    };
}
