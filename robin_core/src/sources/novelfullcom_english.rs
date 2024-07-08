use super::{Novel, NovelChapter};

use crate::utils::{create_progress_bar, INT_FLOAT_REGEX, capitalize};

use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};

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

        let cover_resp = self.client.get(&cover_src).send().await?;
        let cover_bytes = cover_resp.bytes().await?;
        
        Ok((cover_src, cover_bytes.into()))
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