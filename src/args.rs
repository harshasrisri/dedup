use lazy_static::lazy_static;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct DedupOpts {
    /// Activate debug mode
    #[structopt(short, long)]
    pub debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbosity: u8,

    /// File containing list of remote files and hashes
    #[structopt(
        short = "R",
        long = "remote-list",
        parse(from_os_str),
        conflicts_with = "remote_path",
        requires = "hash"
    )]
    pub remote_list: Option<PathBuf>,

    /// Remote path to use as a reference to filter duplicates in local
    #[structopt(
        short = "r",
        long = "remote-path",
        parse(from_os_str),
        conflicts_with = "remote_list"
    )]
    pub remote_path: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[structopt(
        short = "l",
        long = "local-path",
        parse(from_os_str),
        default_value = "."
    )]
    pub local_path: PathBuf,

    /// Type of Hashing algorigthm to use for checksumming.
    #[structopt(short = "H", long, requires = "remote_list")]
    pub hash: Option<String>,

    /// Performs a dry run by default. Use this option to commit file deletions
    #[structopt(short, long)]
    pub commit: bool,
}

lazy_static! {
    #[derive(Debug)]
    pub static ref CLI_OPTS: DedupOpts = DedupOpts::from_args();
}
