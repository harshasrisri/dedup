use std::path::PathBuf;

#[derive(StructOpt, Debug)]
pub struct DedupOpts {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// File containing list of remote files and hashes
    #[structopt(short = "R", long = "remote-list", parse(from_os_str), conflicts_with = "remote_path")]
    pub remote_list: Option<PathBuf>,

    /// Remote path to use as a reference to filter duplicates in local
    #[structopt(short = "r", long = "remote-path", parse(from_os_str), required_unless = "remote_list")]
    pub remote_path: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[structopt(short = "l", long = "local-path", parse(from_os_str), default_value = ".")]
    pub local_path: PathBuf,

    /// Type of Hashing algorigthm to use for checksumming.
    #[structopt(short = "H", long = "hash-algo", default_value = "Md5")]
    pub hash_algo: String,

    /// Performs a dry run by default. Use this option to commit file deletions
    #[structopt(short = "c", long = "commit-delete")]
    pub commit: bool,
}
