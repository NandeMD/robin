use std::path::PathBuf;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};

use clap::Parser;
use zip::{ZipWriter, CompressionMethod, write::FileOptions};
use robin_core::sources::Serie;
use robin_core::matcher::match_manga;

mod args;
use args::{App, Commands};

mod utils;
use utils::copy_dir_all;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::parse();

    match &app.command {
        Commands::Manga { compress, url } => {
            let url = url;
            let mut source = match_manga(url.clone()).await?;
            source.find_chapters().await;

            let info = source.info().clone();
            let manga_name = info.iter()
                .find(|inf| {
                    inf.0 == "name"
                })
                .map(|uwu| {
                    uwu.1.clone()
                })
                .unwrap();
            
            println!("Found manga!\n\n{}\n\nStarting download!", source.format_info(&info));
            let temp = source.download(app.concurrent_chapters).await?;

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
                        .filter(|e| {e.file_type().is_file()})
                    {
                        let mut entry_file = File::open(&ent.path())?;
                        let entry_file_name = ent.path().strip_prefix(&temp.path())?;

                        println!("compressing: {}", entry_file_name.display());

                        zipper.start_file(entry_file_name.to_str().unwrap(), zip_options)?;

                        let mut buffer: Vec<u8> = Vec::new();
                        entry_file.read_to_end(&mut buffer)?;

                        zipper.write_all(&buffer)?;
                    }

                    zipper.finish()?;

                    println!("Done!");

                },
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

