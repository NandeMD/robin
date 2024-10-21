use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::Parser;
use robin_cli_core::matcher::{match_manga, match_novel};
use robin_cli_core::sources::{Novel, Serie};
use robin_cli_core::utils::create_progress_bar;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

use epub_builder::{EpubBuilder, EpubVersion, ZipLibrary};

mod args;
use args::{App, Commands, NovelFormat};

mod utils;
use utils::{copy_dir_all, find_in_info};

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
            let mut source = match_manga(url.clone(), app.proxy.clone()).await?;
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
        Commands::Novel {
            url,
            filter,
            format,
        } => {
            let mut source = match_novel(url.clone(), app.proxy.clone()).await?;
            source.find_chapters().await;
            source.filter_chapters(filter.clone())?;

            let info = source.info().clone();
            let novel_name = info
                .iter()
                .find(|inf| inf.0 == "title")
                .map(|uwu| uwu.1.clone())
                .unwrap();

            println!(
                "Found novel!\n\n{}\n\nStarting download!",
                source.format_info(&info)
            );

            let info = info
                .clone()
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect::<Vec<(String, String)>>();

            let temp = source.download(app.concurrent_chapters).await?;

            match format {
                NovelFormat::Txt => {
                    let output_folder = PathBuf::from(&app.output_folder);
                    let destination = output_folder.join(novel_name);

                    println!("Copying files to: {}", destination.display());

                    create_dir_all(&destination)?;
                    copy_dir_all(&temp, destination)?;
                }
                NovelFormat::Epub => {
                    let mut pbar =
                        create_progress_bar(source.chapters().len() as u64, "Adding files: ");

                    // get all full file paths in the temp directory as &str
                    let mut files = walkdir::WalkDir::new(&temp.path())
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .map(|e| e.path().to_str().unwrap().to_string())
                        .collect::<Vec<String>>();
                    files.sort_by(|a, b| natord::compare(&a, &b));

                    // find the cover image
                    let cover = files.iter().find(|f| f.contains("cover")).unwrap();

                    // open cover image and convert to bytes
                    let mut cover_file = File::open(cover)?;
                    let mut cover_bytes = Vec::new();
                    cover_file.read_to_end(&mut cover_bytes)?;

                    // get the cover image extension and turn into a mimetype
                    let cover_ext = cover.split('.').last().unwrap();
                    let cover_mimetype = match cover_ext {
                        "jpg" | "jpeg" => "image/jpeg",
                        "png" => "image/png",
                        "gif" => "image/gif",
                        _ => "image/jpeg",
                    };

                    let mut book_builder = EpubBuilder::new(ZipLibrary::new().unwrap())
                        .unwrap()
                        .epub_version(EpubVersion::V30)
                        .metadata("title", find_in_info(&info, "title").unwrap())
                        .unwrap()
                        .metadata("author", find_in_info(&info, "author").unwrap())
                        .unwrap()
                        .metadata(
                            "alternative names",
                            find_in_info(&info, "alternative names").unwrap(),
                        )
                        .unwrap()
                        .metadata("genres", find_in_info(&info, "genres").unwrap())
                        .unwrap()
                        .metadata("source", find_in_info(&info, "source").unwrap())
                        .unwrap()
                        .metadata("status", find_in_info(&info, "status").unwrap())
                        .unwrap()
                        .add_cover_image(cover, cover_bytes.as_slice(), cover_mimetype)
                        .unwrap();

                    unimplemented!()
                }
            }
        }
    }

    Ok(())
}
