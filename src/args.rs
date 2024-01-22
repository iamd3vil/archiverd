use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[clap(author = "Sarat Chandra", version, about, long_about = None)]
pub struct Args {
    // The directory to process
    #[clap(short, long)]
    pub directory: String,

    /// Maximum number of archived files to keep
    #[clap(short = 'n', long)]
    pub max_files: Option<usize>,

    #[clap(short, long)]
    pub exclude: Option<String>,

    /// Only keep files with minimum file size. Example: 1 MB, 1 GB
    #[clap(short, long)]
    pub keep_min_size: Option<String>,

    #[clap(long)]
    pub disable_watch: bool,
}
