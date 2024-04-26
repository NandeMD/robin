use super::*;
use futures::StreamExt;
use reqwest::{Client, ClientBuilder};
use scraper::{selectable::Selectable, Html, Selector};
use crate::utils::capitalize;

use tempfile::{tempdir, TempDir};
use tokio::fs::{File, create_dir};
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
        let chapter_name_selector = Selector::parse("div:nth-child(1) > div:nth-child(1) > a:nth-child(1) > span:nth-child(1)").unwrap();
        let chapter_date_selector = Selector::parse("div:nth-child(1) > div:nth-child(1) > a:nth-child(1) > span:nth-child(2)").unwrap();
        let chapter_url_selector = Selector::parse("div:nth-child(1) > div:nth-child(1) > a:nth-child(1)").unwrap();

        for ch in self.data.select(&chapters_list_selector) {
            let name: String = ch.select(&chapter_name_selector)
                .next()
                .unwrap()
                .text()
                .collect();

            let date: String = ch.select(&chapter_date_selector)
                .next()
                .unwrap()
                .text()
                .collect();

            let url: String = ch.select(&chapter_url_selector)
                .next()
                .unwrap()
                .attr("href")
                .unwrap()
                .into();

            self.chapters.push( ShijieTurkishChapter { date, name, url, page_urls: Vec::new(), page_data: Vec::new() } );
        }
    }

    async fn download(&mut self, n_sim: usize) -> anyhow::Result<TempDir> {
        let tmpdir = tempdir()?;
        let tmp_path = tmpdir.path();
        let client = &self.client;
        let stream = futures::stream::iter(
            self.chapters.iter_mut().map(|c| async move {
                let dir_path = tmp_path.join(&c.name);
                create_dir(&dir_path).await?;
                
                c.download(client).await?;

                for page in &c.page_data {
                    let filename = page.0.split("/").last().unwrap();
                    let temp_page_path = dir_path.join(filename);

                    let mut f = File::create(temp_page_path).await?;
                    f.write_all(page.1.as_ref()).await?;
                }

                anyhow::Ok(())
            })
        ).buffered(n_sim);

        let results = stream.collect::<Vec<_>>().await;

        for r in results {
            r?
        }
        Ok(tmpdir)
    }

    fn chapter_count(&self) -> usize {
        self.chapters.len()
    }

    fn info(&self) -> Vec<(&str, String)> {
        // Selectors for Info
        let name_selector = Selector::parse("h1.entry-title").unwrap();
        let alternative_selector = Selector::parse(".entry-content > p").unwrap();
        let tags_selector = Selector::parse(".mgen > a").unwrap();
        let first_chapter_selector = Selector::parse(".epcurfirst").unwrap();
        let last_chapter_selector = Selector::parse(".epcurlast").unwrap();

        let name: String = self.data.select(&name_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let alternative_holder = self.data.select(&alternative_selector).next();
        let mut alternative: String = String::new();

        if let Some(alt) = alternative_holder {
            alternative = alt.text().collect()
        };

        let tags = self.data.select(&tags_selector)
            .map(|t| { t.text().collect::<String>() })
            .collect::<Vec<String>>()
            .join(", ");

        let first_chapter: String = self.data.select(&first_chapter_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let last_chapter: String = self.data.select(&last_chapter_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let mut map:Vec<(&str, String)> = Vec::new();
        map.push(("name", name));
        map.push(("alt. name", alternative));
        map.push(("categories", tags));
        map.push(("first chapter", first_chapter));
        map.push(("last chapter", last_chapter));
        map.push(("chapter count", format!("{}", self.chapter_count())));

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
}

#[derive(Debug)]
pub struct ShijieTurkishChapter {
    pub date: String,
    pub name: String,
    pub url: String,
    
    page_urls: Vec<String>,
    page_data: Vec<(String, Vec<u8>)>
}

impl Chapter for ShijieTurkishChapter {
    async fn fetch(&mut self, c: &Client) -> anyhow::Result<()> {
        let stream = futures::stream::iter(
            self.page_urls.iter().map(|uri| async move {
                let p = c.get(uri).send().await;

                p
            }))
        .   buffered(10);
        
        let results = stream.collect::<Vec<_>>().await;

        for (i, res) in results.into_iter().enumerate() {
            let response = res?;

            let response_url = response.url().to_string();
            let file_ext = response_url.split(".").last().unwrap();

            self.page_data.push(
                (format!("{:0>4}.{}", i, file_ext), response.bytes().await?.into())
            );
        }

        Ok(())
    }

    async fn search_image_urls(&mut self, c: &Client) -> anyhow::Result<()> {
        let ts_getter_selector = Selector::parse(".wrapper > script:nth-child(2)").unwrap();

        let page = c.get(&self.url).send().await?;
        let ptext = page.text().await?;

        let data = Html::parse_document(&ptext.as_str());

        let inner_script: String = data.select(&ts_getter_selector)
            .next()
            .unwrap()
            .text()
            .collect();

        let json_part = &inner_script[14..inner_script.len()-2];
        let deserred:SourcesDeser = serde_json::from_str(json_part)?;

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
}

#[derive(Deserialize)]
struct SourcesDeser {
    pub sources: Vec<SourceDeser>
}

#[derive(Deserialize)]
struct SourceDeser {
    pub images: Vec<String>
}