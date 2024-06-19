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
    #[arg(short, long, default_value_t = 1)]
    pub concurrent_chapters: usize,

    /// Proxy URL. Example: http://uwu.com:8080
    #[arg(short, long, default_value_t = String::new())]
    pub proxy: String,
}

#[derive(Clone, Subcommand)]
pub enum Commands {
    Manga {
        #[arg(short, long, default_value_t = false)]
        compress: bool,

        /// Chapter filter. Example 1: 10:100 Example 2: 20.5:100.3  (Note: both numbers included)
        #[arg(short, long, default_value_t = String::new())]
        filter: String,

        /// URL of the source content
        url: String,
    },

    Novel {
        // URL of the source content
        url: String,
    },
}
