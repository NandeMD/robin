use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use robin_cli_core::matcher::match_manga;
use robin_cli_core::sources::Serie;
use robin_cli_core::utils::create_progress_bar;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

mod args;
use args::{App, Commands};

mod utils;
use utils::copy_dir_all;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::parse();

    match &app.command {
        Commands::Manga {
            compress,
            url,
            filter,
        } => {
            let url = url;
            let mut source = match_manga(url.clone()).await?;
            source.find_chapters().await;
            source.filter_chapters(filter.clone())?;

            let info = source.info().clone();
            let manga_name = info
                .iter()
                .find(|inf| inf.0 == "title")
                .map(|uwu| uwu.1.clone())
                .unwrap();

            println!(
                "Found manga!\n\n{}\n\nStarting download!",
                source.format_info(&info)
            );

            let temp = source.download(app.concurrent_chapters).await?;

            let mut pbar = create_progress_bar(source.chapter_count() as u64, "Adding files: ");

            match compress {
                true => {
                    let output_folder = PathBuf::from(&app.output_folder);
                    let destination = output_folder.join(format!("{}.zip", manga_name));
                    println!("Destination set to: {}", destination.display());
                    let f = File::create(destination)?;

                    let mut zipper = ZipWriter::new(f);
                    let zip_options = FileOptions::default()
                        .compression_method(CompressionMethod::Bzip2)
                        .compression_level(Some(9))
                        .large_file(true);

                    for ent in walkdir::WalkDir::new(&temp.path())
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                    {
                        let mut entry_file = File::open(&ent.path())?;
                        let entry_file_name = ent.path().strip_prefix(&temp.path())?;

                        zipper.start_file(entry_file_name.to_str().unwrap(), zip_options)?;

                        let mut buffer: Vec<u8> = Vec::new();
                        entry_file.read_to_end(&mut buffer)?;

                        zipper.write_all(&buffer)?;

                        pbar.inc();
                    }

                    pbar.message("Compressing files...");

                    zipper.finish()?;

                    pbar.finish_print("Compressed!");
                }
                false => {
                    let output_folder = PathBuf::from(&app.output_folder);
                    let destination = output_folder.join(manga_name);

                    println!("Copying files to: {}", destination.display());

                    create_dir_all(&destination)?;
                    copy_dir_all(&temp, destination)?;
                }
            }
        }
    }

    Ok(())
}
