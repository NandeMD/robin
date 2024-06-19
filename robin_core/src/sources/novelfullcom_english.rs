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
        let last_page_url = self.data.select(&last_page_selector).next().unwrap().value().attr("href").unwrap();
        let last_page_number = last_page_url
            .rsplit_once("=")
            .unwrap()
            .1
            .parse::<u64>()
            .unwrap();

        // let mut pb = create_progress_bar(last_page_number, "Fetching chapters: ");

        for i in 1..=last_page_number {
            let page_url = format!("{}?page={}", self.url, i);
            let page = self.client.get(&page_url).send().await.unwrap().text().await.unwrap();
            let data = Html::parse_document(&page);

            let chapter_link_selector = Selector::parse("div.col-sm-6:nth-child(1) > ul:nth-child(1) > li > a").unwrap();

            for ch_link in data.select(&chapter_link_selector) {
                let title = ch_link.text().collect::<String>();
                let href = ch_link.value().attr("href").unwrap();

                let ch_url: String;

                if href.starts_with("http") {
                    ch_url = href.to_string();
                } else {
                    ch_url = format!("{}{}", BASE_URL, href);
                }

                println!("{}: {}", title, ch_url);

                self.chapters.push(NovelFullComChapter {
                    title,
                    url: ch_url
                });

                if i == last_page_number {
                    // pb.finish_print(format!("{} chapters fetched!", self.chapters.len()).as_str());
                    println!("{} chapters fetched!", self.chapters.len());
                    break;
                }
            }
        }
    }

    async fn get_cover(&self) -> anyhow::Result<(String, Vec<u8>)> {
        unimplemented!()
    }
}

pub struct NovelFullComChapter {
    title: String,
    url: String,
}

impl NovelChapter for NovelFullComChapter {}