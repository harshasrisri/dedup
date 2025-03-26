use crate::digest::DigestKind;
use clap::Parser;
use lazy_static::lazy_static;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct DedupOpts {
    /// Remote path to use as a reference to filter duplicates in local
    #[arg(short, long, conflicts_with = "input_file")]
    pub remote_path: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,

    /// Type of digest to use to parse/generate digest-mode analysis
    #[arg(short, long, default_value = "sha1")]
    pub digest: DigestKind,

    /// File containing digest-mode analysis used to dedup files in local_path
    #[arg(short, long, conflicts_with = "remote_path", requires = "digest")]
    pub input_file: Option<PathBuf>,

    /// File to write the output of digest-mode analysis
    #[arg(
        short,
        long,
        conflicts_with_all = [ "remote_path", "input_file" ],
        requires = "digest",
        default_value = "dedup.out"
    )]
    pub output_file: PathBuf,

    /// Performs a dry run by default. Use this option to commit file deletions
    #[arg(short, long)]
    pub commit: bool,

    /// Flag count for log verbosity (info(1), debug(2), trace(3)) [default: warn(0)]
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,
}

lazy_static! {
    #[derive(Debug)]
    pub static ref CLI_OPTS: DedupOpts = DedupOpts::parse();
}
