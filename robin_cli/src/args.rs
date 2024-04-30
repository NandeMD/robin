use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct App {
    #[command(subcommand)]
    pub command: Commands,

    /// Where should downloaded serie stay huh?
    #[arg(short, long)]
    pub output_folder: String,

    /// Number of chapters that will be downloaded at the same time
    #[arg(short, long, default_value_t = 10)]
    pub concurrent_chapters: usize,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    Manga {
        #[arg(short, long, default_value_t = false)]
        tachiyomi: bool,

        /// URL of the source content
        url: String,
    }
}

