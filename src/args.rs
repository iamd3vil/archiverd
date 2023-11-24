use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[clap(author = "Sarat Chandra", version, about, long_about = None)]
pub struct Args {
    // The directory to process
    #[clap(short, long)]
    pub directory: String,

    /// Maximum number of files to process
    #[clap(short = 'n', long)]
    pub max_files: Option<usize>,

    #[clap(short, long)]
    pub exclude: Option<String>,

    #[clap(long)]
    pub disable_watch: bool,
}
