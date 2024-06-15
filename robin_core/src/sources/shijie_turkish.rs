use super::*;
use crate::utils::{capitalize, create_progress_bar};
use futures::StreamExt;
use reqwest::{Client, ClientBuilder};
use scraper::{selectable::Selectable, Html, Selector};
use std::sync::{Arc, Mutex};

use tempfile::{tempdir, TempDir};
use tokio::fs::{create_dir, File};
use tokio::io::AsyncWriteExt;

use serde::Deserialize;

#[derive(Debug)]
pub struct ShijieTurkish {
    pub url: String,
    pub client: Client,
    pub data: Html,
    pub chapters: Vec<ShijieTurkishChapter>,
}

impl Serie for ShijieTurkish {
    async fn new(url: String) -> anyhow::Result<ShijieTurkish> {
        let client = ClientBuilder::new()
            .connection_verbose(true)
            .cookie_store(true)
            .deflate(true)
            .gzip(true)
            .brotli(true)
            .build()
            .unwrap();

        let page = client.get(&url).send().await?;
        let data = Html::parse_document(page.text().await?.as_str());

        Ok(ShijieTurkish {
            url,
            client,
            data,
            chapters: Vec::new(),
        })
    }

    async fn find_chapters(&mut self) {
        let chapters_list_selector = Selector::parse("#chapterlist > ul > li").unwrap();
        let chapter_name_selector = Selector::parse(
            "div:nth-child(1) > div:nth-child(1) > a:nth-child(1) > span:nth-child(1)",
        )
        .unwrap();
        let chapter_date_selector = Selector::parse(
            "div:nth-child(1) > div:nth-child(1) > a:nth-child(1) > span:nth-child(2)",
        )
        .unwrap();
        let chapter_url_selector =
            Selector::parse("div:nth-child(1) > div:nth-child(1) > a:nth-child(1)").unwrap();

        for ch in self.data.select(&chapters_list_selector) {
            let name: String = ch
                .select(&chapter_name_selector)
                .next()
                .unwrap()
                .text()
                .collect();

            let date: String = ch
                .select(&chapter_date_selector)
                .next()
                .unwrap()
                .text()
                .collect();

            let url: String = ch
                .select(&chapter_url_selector)
                .next()
                .unwrap()
                .attr("href")
                .unwrap()
                .into();

            self.chapters.push(ShijieTurkishChapter {
                date,
                name,
                url,
                page_urls: Vec::new(),
                page_data: Vec::new(),
            });
        }
    }

    fn chapters(&mut self) -> &mut Vec<impl Chapter> {
        &mut self.chapters
    }

    async fn get_cover(&self) -> anyhow::Result<(String, Vec<u8>)> {
        let cover_selector = Selector::parse(".attachment-").unwrap();
        let cover_url = self
            .data
            .select(&cover_selector)
            .next()
            .unwrap()
            .attr("src")
            .unwrap();
        let cover_url_ext = cover_url.split(".").last().unwrap();

        let cover_response = self.client.get(cover_url).send().await?;
        let cover_bytes = cover_response.bytes().await?;

        Ok((cover_url_ext.to_string(), cover_bytes.into()))
    }

    async fn download(&mut self, n_sim: usize) -> anyhow::Result<TempDir> {
        let tmpdir = tempdir()?;
        let tmp_path = tmpdir.path();
        println!("Temporary directory created to: {}", &tmp_path.display());

        let client = &self.client;
        let chapter_count = self.chapter_count();
        let pbar = Arc::new(Mutex::new(create_progress_bar(
            chapter_count as u64,
            "Downloading: ",
        )));

        // Download cover image and save it to the temporary directory
        let cover_data = self.get_cover().await?;
        let cover_filename = format!("cover.{}", cover_data.0);
        let mut f = File::create(tmp_path.join(cover_filename)).await?;
        f.write_all(cover_data.1.as_ref()).await?;

        // Save details to a json file
        let details = self.details();
        let mut f = File::create(tmp_path.join("details.json")).await?;
        f.write_all(details.as_bytes()).await?;

        // The first map is to clone the current_chapter mutex.
        // There is probably better ways to do it but I'm not sure how to do it
        let stream = futures::stream::iter(
            self.chapters
                .iter_mut()
                .map(|c| {
                    let counter = Arc::clone(&pbar);
                    (c, counter)
                })
                .map(|(c, counter)| async move {
                    let dir_path = tmp_path.join(&c.name);
                    create_dir(&dir_path).await?;

                    c.download(client).await?;

                    for page in &c.page_data {
                        let filename = page.0.split("/").last().unwrap();
                        let temp_page_path = dir_path.join(filename);

                        let mut f = File::create(temp_page_path).await?;
                        f.write_all(page.1.as_ref()).await?;
                    }

                    // Clear page data to save memory
                    c.page_data.clear();

                    // Notify progress
                    let mut counter = counter.lock().unwrap();
                    counter.inc();
                    drop(counter); // Unlock Mutex (counter)

                    anyhow::Ok(())
                }),
        )
        .buffered(n_sim);

        let results = stream.collect::<Vec<_>>().await;

        for r in results {
            r?
        }

        pbar.lock().unwrap().finish_print("Downloaded!");
        Ok(tmpdir)
    }

    fn chapter_count(&self) -> usize {
        self.chapters.len()
    }

    fn info(&self) -> Vec<(&str, String)> {
        // Selectors for Info
        let title_selector = Selector::parse("h1.entry-title").unwrap();
        let author_selector =
            Selector::parse("div.flex-wrap:nth-child(4) > div:nth-child(2) > span:nth-child(2)")
                .unwrap();
        let artist_selector = author_selector.clone();
        let description_selector = Selector::parse(".entry-content > p").unwrap();
        let genres_selector = Selector::parse(".mgen > a").unwrap();
        let first_chapter_selector = Selector::parse(".epcurfirst").unwrap();
        let last_chapter_selector = Selector::parse(".epcurlast").unwrap();
        let status_selector = Selector::parse("div.imptdt:nth-child(1) > i:nth-child(1)").unwrap();

        let title: String = self
            .data
            .select(&title_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let author: String = self
            .data
            .select(&author_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let artist: String = self
            .data
            .select(&artist_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let description_holder = self.data.select(&description_selector).next();
        let mut description: String = String::new();

        if let Some(alt) = description_holder {
            description = alt.text().collect::<String>().replace("\n", " ");
        };

        let genres = self
            .data
            .select(&genres_selector)
            .map(|t| {
                let tt = t.text().collect::<String>();
                let mut buff = String::new();
                buff.push('"');
                buff.push_str(&tt);
                buff.push('"');
                buff
            })
            .collect::<Vec<String>>()
            .join(", ");

        let first_chapter: String = self
            .data
            .select(&first_chapter_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let last_chapter: String = self
            .data
            .select(&last_chapter_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let status: String = self
            .data
            .select(&status_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let mut map: Vec<(&str, String)> = Vec::new();
        map.push(("title", title));
        map.push(("author", author));
        map.push(("artist", artist));
        map.push(("description", description));
        map.push(("genres", genres));
        map.push(("first chapter", first_chapter));
        map.push(("last chapter", last_chapter));
        map.push(("chapter count", format!("{}", self.chapter_count())));
        map.push(("status", status));

        map
    }

    fn details(&self) -> String {
        let info = self.info();

        let title = info.iter().find(|inf| inf.0 == "title").unwrap().1.clone();
        let author = info.iter().find(|inf| inf.0 == "author").unwrap().1.trim();
        let artist = info.iter().find(|inf| inf.0 == "artist").unwrap().1.trim();
        let description = info
            .iter()
            .find(|inf| inf.0 == "description")
            .unwrap()
            .1
            .trim();
        let genres = info.iter().find(|inf| inf.0 == "genres").unwrap().1.clone();
        let status_holder = info.iter().find(|inf| inf.0 == "status").unwrap().1.clone();
        let status = match status_holder.as_str() {
            "Devam Ediyor" => "1",
            "Final" => "2",
            "Sezon Finali" | "Askıda" => "6",
            _ => "0",
        };
        format!(
            r#"
        {{
            "title": "{}",
            "author": "{}",
            "artist": "{}",
            "description": "{}",
            "genre": [{}],
            "status": "{}"
        }}
        "#,
            title, author, artist, description, genres, status
        )
    }

    fn format_info(&self, info: &Vec<(&str, String)>) -> String {
        let mut buff = String::new();

        for (k, v) in info {
            buff.push_str(&capitalize(k));
            buff.push_str(": ");
            buff.push_str(&v);
            buff.push('\n');
        }

        buff
    }
}

#[derive(Debug)]
pub struct ShijieTurkishChapter {
    pub date: String,
    pub name: String,
    pub url: String,

    page_urls: Vec<String>,
    page_data: Vec<(String, Vec<u8>)>,
}

impl Chapter for ShijieTurkishChapter {
    async fn fetch(&mut self, c: &Client) -> anyhow::Result<()> {
        let stream = futures::stream::iter(self.page_urls.iter().map(|uri| async move {
            let p = c.get(uri).send().await;

            p
        }))
        .buffered(10);

        let results = stream.collect::<Vec<_>>().await;

        for (i, res) in results.into_iter().enumerate() {
            let response = res?;

            let response_url = response.url().to_string();
            let file_ext = response_url.split(".").last().unwrap();

            self.page_data.push((
                format!("{:0>4}.{}", i, file_ext),
                response.bytes().await?.into(),
            ));
        }

        Ok(())
    }

    async fn search_image_urls(&mut self, c: &Client) -> anyhow::Result<()> {
        let ts_getter_selector = Selector::parse(".wrapper > script:nth-child(2)").unwrap();

        let page = c.get(&self.url).send().await?;
        let ptext = page.text().await?;

        let data = Html::parse_document(&ptext.as_str());

        let inner_script: String = data
            .select(&ts_getter_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let json_part = &inner_script[14..inner_script.len() - 2];
        let deserred: SourcesDeser = serde_json::from_str(json_part)?;

        for img in &deserred.sources[0].images {
            self.page_urls.push(img.clone());
        }

        Ok(())
    }
    fn page_count(&self) -> usize {
        self.page_urls.len()
    }

    async fn download(&mut self, c: &Client) -> anyhow::Result<()> {
        self.search_image_urls(c).await?;
        self.fetch(c).await?;

        Ok(())
    }

    fn info(&self) -> Vec<(&str, String)> {
        let mut map: Vec<(&str, String)> = Vec::new();

        map.push(("name", self.name.clone()));
        map.push(("release date", self.date.clone()));
        map.push(("source", self.url.clone()));
        map.push(("page count", format!("{}", self.page_count())));

        map
    }

    fn format_info(&self, info: &Vec<(&str, String)>) -> String {
        let mut buff = String::new();

        for (k, v) in info {
            buff.push_str(&capitalize(k));
            buff.push_str(": ");
            buff.push_str(&v);
            buff.push('\n');
        }

        buff
    }

    fn chapter_num(&self) -> f64 {
        let num = *self
            .name
            .split(" ")
            .into_iter()
            .map(|word| word.replace(",", "."))
            .filter_map(|s| s.parse::<f64>().ok())
            .collect::<Vec<f64>>()
            .first()
            .unwrap();

        // Round to 2 decimal points. Example: 123.45
        (num * 100.0).round() / 100.0
    }
}

#[derive(Deserialize)]
struct SourcesDeser {
    pub sources: Vec<SourceDeser>,
}

#[derive(Deserialize)]
struct SourceDeser {
    pub images: Vec<String>,
}
