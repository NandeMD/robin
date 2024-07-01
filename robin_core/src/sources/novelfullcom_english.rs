use super::{Novel, NovelChapter};

use crate::utils::create_progress_bar;

use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};

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
                    url: ch_url
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
}

impl NovelChapter for NovelFullComChapter {}