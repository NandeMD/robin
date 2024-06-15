use crate::sources::*;

pub async fn match_manga(url: String, proxy: String) -> anyhow::Result<impl Serie> {
    if url.starts_with("https://shijiescans.com") {
        return Ok(shijie_turkish::ShijieTurkish::new(url, proxy).await?);
    } else {
        return Err(anyhow::Error::msg("Unsupported Source"));
    }
}