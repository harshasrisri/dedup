use crate::hash::HashMode;
use clap::Parser;
use lazy_static::lazy_static;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct DedupOpts {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// Remote path to use as a reference to filter duplicates in local
    #[arg(short = 'r', long = "remote-path", conflicts_with = "remote_list")]
    pub remote_path: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short = 'l', long = "local-path", default_value = ".")]
    pub local_path: PathBuf,

    /// Type of Hashing algorigthm to use for checksumming.
    #[arg(short = 'H', long, default_value = "sha1")]
    pub hash: HashMode,

    /// File containing list of remote files and hashes
    #[arg(
        short = 'R',
        long = "remote-list",
        conflicts_with = "remote_path",
        requires = "hash"
    )]
    pub remote_list: Option<PathBuf>,

    /// File to write the output of hash mode analysis
    #[arg(
        short = 'o',
        long = "output-file",
        conflicts_with_all = [ "remote_path", "remote_list" ],
        requires = "hash",
        default_value = "dedup.out"
    )]
    pub output_file: PathBuf,

    /// Performs a dry run by default. Use this option to commit file deletions
    #[arg(short, long)]
    pub commit: bool,
}

lazy_static! {
    #[derive(Debug)]
    pub static ref CLI_OPTS: DedupOpts = DedupOpts::parse();
}
