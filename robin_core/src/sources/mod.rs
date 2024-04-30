use std::future::Future;

use reqwest::Client;
use tempfile::TempDir;

pub mod shijie_turkish;

pub trait Serie {
    fn new(url: String) -> impl Future<Output = anyhow::Result<impl Serie>> + Sync;
    fn find_chapters(&mut self) -> impl Future<Output = ()>;
    fn get_cover(&self) -> impl Future<Output = anyhow::Result<(String, Vec<u8>)>>;
    // n_sim is number of chapters that will be downloaded in parallel
    fn download(&mut self, n_sim: usize) -> impl Future<Output = anyhow::Result<TempDir>>; 

    fn chapter_count(&self) -> usize;
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
}