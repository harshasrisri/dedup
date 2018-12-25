use std::path::PathBuf;

/// Program to find duplicate files and take care of them
#[derive(StructOpt, Debug)]
#[structopt(name = "dedup")]
pub struct DedupOpts {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// File containing list of remote files and hashes
    #[structopt(short = "R", long = "remote-list", parse(from_os_str))]
    pub remote_list: PathBuf,

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
