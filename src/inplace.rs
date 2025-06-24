use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Dedups files in a folder in-place
pub struct InPlace {
    /// Local Path containing files that need to be checked for duplicates in-place
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,
}
