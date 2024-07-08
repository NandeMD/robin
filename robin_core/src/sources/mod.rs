use std::future::Future;

use reqwest::Client;
use tempfile::TempDir;

pub mod shijie_turkish;
pub mod novelfullcom_english;

pub trait Serie {
    fn new(url: String, proxy: String) -> impl Future<Output = anyhow::Result<impl Serie>> + Sync;
    fn find_chapters(&mut self) -> impl Future<Output = ()>;
    fn get_cover(&self) -> impl Future<Output = anyhow::Result<(String, Vec<u8>)>>;
    // n_sim is number of chapters that will be downloaded in parallel
    fn download(&mut self, n_sim: usize) -> impl Future<Output = anyhow::Result<TempDir>>;

    fn parse_chapter_filter(&self, a: String) -> anyhow::Result<Option<(f64, f64)>> {
        if a.is_empty() {
            return Ok(None);
        }

        let splitted = a.split(":").collect::<Vec<&str>>();

        let lower_bound: f64 = splitted[0].parse()?;
        let upper_bound: f64 = splitted[1].parse()?;

        Ok(Some((lower_bound, upper_bound)))
    }

    fn filter_chapters(&mut self, filter_param: String) -> anyhow::Result<()> {
        if let Some((lb, ub)) = self.parse_chapter_filter(filter_param)? {
            self.chapters()
                .retain(|c| c.chapter_num() >= lb && c.chapter_num() <= ub);
        }
        Ok(())
    }

    fn chapter_count(&self) -> usize;
    fn chapters(&mut self) -> &mut Vec<impl Chapter>;
    fn info(&self) -> Vec<(&str, String)>;
    fn details(&self) -> String;
    fn format_info(&self, info: &Vec<(&str, String)>) -> String;
}

pub trait Chapter {
    fn fetch(&mut self, c: &Client) -> impl Future<Output = anyhow::Result<()>>;
    fn search_image_urls(&mut self, c: &Client) -> impl Future<Output = anyhow::Result<()>>;
    fn download(&mut self, c: &Client) -> impl Future<Output = anyhow::Result<()>>;

    fn page_count(&self) -> usize;
    fn info(&self) -> Vec<(&str, String)>;
    fn format_info(&self, info: &Vec<(&str, String)>) -> String;
    fn chapter_num(&self) -> f64;
}


pub trait Novel {
    fn new(url: String, proxy: String) -> impl Future<Output = anyhow::Result<impl Novel>> + Sync;
    fn find_chapters(&mut self) -> impl Future<Output = ()>;
    fn get_cover(&self) -> impl Future<Output = anyhow::Result<(String, Vec<u8>)>>;
}

pub trait NovelChapter {
    fn download(&mut self, c: &Client) -> impl Future<Output = anyhow::Result<()>>;

    fn info(&self) -> Vec<(&str, String)>;
    fn format_info(&self, info: &Vec<(&str, String)>) -> String;
    fn chapter_num(&self) -> f64;
}