use crate::digest::DigestKind;
use clap::{Parser, Args, Subcommand};
use lazy_static::lazy_static;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct DedupOpts {
    /// Performs a dry run by default. Use this option to commit file deletions
    #[arg(short, long)]
    pub commit: bool,

    /// Flag count for log verbosity (info(1), debug(2), trace(3)) [default: warn(0)]
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    #[command(subcommand)]
    pub mode: OperatingMode,
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum OperatingMode {
    Analyze(Analyze),
    Remote(Remote),
    Local(Local),
    InPlace(InPlace),
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Analyzes path and writes data to file to be used elsewhere
pub struct Analyze {
    /// File to write the output of digest-mode analysis
    #[arg(
        short,
        long,
        requires = "digest",
        default_value = "dedup.out"
    )]
    pub output_file: PathBuf,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,

    /// Type of digest to use to parse/generate digest-mode analysis
    #[arg(short, long, default_value = "sha1")]
    pub digest: DigestKind,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Uses analysis from a file to dedup files
pub struct Remote {
    /// File containing digest-mode analysis used to dedup files in local_path
    #[arg(short, long, requires = "digest")]
    pub input_file: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,

    /// Type of digest to use to parse/generate digest-mode analysis
    #[arg(short, long, default_value = "sha1")]
    pub digest: DigestKind,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Dedups files in one folder while referencing another folder
pub struct Local {
    /// Path to use as a reference to filter duplicates in local
    #[arg(short, long, conflicts_with = "input_file")]
    pub reference_path: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,
}

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Dedups files in a folder in-place
pub struct InPlace {
    /// Local Path containing files that need to be checked for duplicates in-place
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,
}

lazy_static! {
    #[derive(Debug)]
    pub static ref CLI_OPTS: DedupOpts = DedupOpts::parse();
}
