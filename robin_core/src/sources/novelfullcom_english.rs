use super::{Novel, NovelChapter};

use crate::utils::{create_progress_bar, INT_FLOAT_REGEX, capitalize};

use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};

use std::sync::{Arc, Mutex};
use futures::StreamExt;
use tempfile::{tempdir, TempDir};
use tokio::fs::{create_dir, File};
use tokio::io::AsyncWriteExt;

use regex::Regex;

const BASE_URL: &str = "https://novelfull.com";

pub struct NovelFullCom {
    pub url: String,
    pub client: Client,
    pub data: Html,
    pub chapters: Vec<NovelFullComChapter>,
}

impl Novel for NovelFullCom {
    async fn new(url: String, proxy: String) -> anyhow::Result<NovelFullCom> {
        let mut client = ClientBuilder::new()
            .connection_verbose(true)
            .cookie_store(true)
            .deflate(true)
            .gzip(true)
            .brotli(true);

        if !proxy.is_empty() {
            client = client.proxy(reqwest::Proxy::all(proxy)?);
        }

        let client = client.build()?;

        let page = client.get(&url).send().await?.text().await?;
        let data = Html::parse_document(&page);
        
        Ok(NovelFullCom {
            url,
            client,
            data,
            chapters: Vec::new()
        })
    }

    async fn find_chapters(&mut self) {
        let last_page_selector = Selector::parse(".last > a:nth-child(1)").unwrap();
        let chapter_link_selector = Selector::parse("div.col-sm-6 > ul:nth-child(1) > li > a").unwrap();
        let last_page_url = self.data.select(&last_page_selector).next().unwrap().value().attr("href").unwrap();
        let last_page_number = last_page_url
            .rsplit_once("=")
            .unwrap()
            .1
            .parse::<u64>()
            .unwrap();

        // get the total chapter count
        let last_page_text = self.client.get(format!("{}/{}", BASE_URL, last_page_url))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let last_page_data = Html::parse_document(&last_page_text);
        let last_page_chapter_count = last_page_data.select(&chapter_link_selector).count();

        let total = last_page_chapter_count as u64 + ((last_page_number - 1) * 50);


        let mut pb = create_progress_bar(total, "Finding chapters: ");

        for i in 1..=last_page_number {
            let page_url = format!("{}?page={}", self.url, i);
            let page = self.client.get(&page_url).send().await.unwrap().text().await.unwrap();
            let data = Html::parse_document(&page);

            for ch_link in data.select(&chapter_link_selector) {
                let title = ch_link.text().collect::<String>();
                let href = ch_link.value().attr("href").unwrap();

                let ch_url: String;

                if href.starts_with("http") {
                    ch_url = href.to_string();
                } else {
                    ch_url = format!("{}/{}", BASE_URL, href);
                }

                self.chapters.push(NovelFullComChapter {
                    title,
                    url: ch_url,
                    content: String::new()
                });

                pb.inc();

                if i == last_page_number {
                    break;
                }
            }
        }

        pb.finish();
    }

    async fn get_cover(&self) -> anyhow::Result<(String, Vec<u8>)> {
        let cover_selector = Selector::parse(".book > img:nth-child(1)").unwrap();
        let cover_src = format!("{BASE_URL}{}", 
            self.data.select(&cover_selector)
                .next()
                .unwrap()
                .attr("src")
                .unwrap()
        );

        let cover_url_ext = cover_src.split(".").last().unwrap();
        let cover_resp = self.client.get(&cover_src).send().await?;
        let cover_bytes = cover_resp.bytes().await?;
        
        Ok((cover_url_ext.into(), cover_bytes.into()))
    }

    async fn download(&mut self, n_sim: usize) -> anyhow::Result<TempDir> {
        let tmpdir = tempdir()?;
        let tmp_path = tmpdir.path();
        println!("Temporary directory created at: {:?}", &tmp_path);
        
        let client = &self.client;
        let chapter_count = self.chapters.len();
        let pbar = Arc::new(Mutex::new(create_progress_bar(chapter_count as u64, "Downloading: ")));

        // Download cover image and save it to the temporary directory
        let (cover_src, cover_bytes) = self.get_cover().await?;
        let cover_filename = format!("cover.{}", cover_src);
        let mut f = File::create(tmp_path.join(&cover_filename)).await?;
        f.write_all(&cover_bytes).await?;

        // Download chapters
        

        Ok(tmpdir)
    }

    fn chapters(&mut self) -> &mut Vec<impl super::NovelChapter> {
        &mut self.chapters
    }

    fn info(&self) -> Vec<(&str, String)> {
        let mut buff: Vec<(&str, String)> = Vec::new();

        let title_selector = Selector::parse("div.col-xs-12:nth-child(3) > h3:nth-child(1)").unwrap();
        let author_selector = Selector::parse(".info > div:nth-child(1) > a").unwrap();
        let alternative_names_selector = Selector::parse(".info > div:nth-child(2)").unwrap();
        let genres_selector = Selector::parse(".info > div:nth-child(3) > a").unwrap();
        let source_selector = Selector::parse(".info > div:nth-child(4)").unwrap();
        let status_selector = Selector::parse(".info > div:nth-child(5) > a").unwrap();

        let title = self.data.select(&title_selector).next().unwrap().text().collect::<String>();

        let author = self.data.select(&author_selector)
            .map(|t| {
                let tt = t.text().collect::<String>();
                tt
            })
            .collect::<Vec<String>>()
            .join(", ");

        let alternative_names = self.data.select(&alternative_names_selector).next().unwrap().text().collect::<String>();
        
        let genres = self.data.select(&genres_selector)
            .map(|t| {
                let tt = t.text().collect::<String>();
                tt
            })
            .collect::<Vec<String>>()
            .join(", ");

        let source = self.data.select(&source_selector).next().unwrap().text().collect::<String>();

        let status = self.data.select(&status_selector).next().unwrap().text().collect::<String>();

        buff.push(("title", title));
        buff.push(("author", author));
        buff.push(("alternative names", alternative_names));
        buff.push(("genres", genres));
        buff.push(("source", source));
        buff.push(("status", status));
        
        buff
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

pub struct NovelFullComChapter {
    title: String,
    url: String,
    content: String,
}

impl NovelChapter for NovelFullComChapter {
    async fn download(&mut self, c: &Client) -> anyhow::Result<()> {
        let page = c.get(&self.url).send().await?.text().await?;
        let data = Html::parse_document(&page);

        let content_selector = Selector::parse("#chapter-content > p").unwrap();
        
        let content = data.select(&content_selector)
            .map(|p| p.text().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n\n");

        self.content = content;

        Ok(())
    }

    fn chapter_num(&self) -> f64 {
        let re = Regex::new(INT_FLOAT_REGEX).unwrap();

        // find number regex in title
        let num = re.find(&self.title).unwrap().as_str();

        num.parse().unwrap()
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

    fn info(&self) -> Vec<(&str, String)> {
        let mut buff: Vec<(&str, String)> = Vec::new();

        buff.push(("title", self.title.clone()));
        buff.push(("source", self.url.clone()));

        // calculate word count
        let word_count = self.content.split_whitespace().count();

        // calculate character_count
        let character_count = self.content.chars().count();

        buff.push(("word count", word_count.to_string()));
        buff.push(("character count", character_count.to_string()));

        buff
    }
}