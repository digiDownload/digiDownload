use crate::scraper::base_scraper::BaseScraper;
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait Scraper: BaseScraper + Sync + Send + Debug {
    async fn fetch_page_pdf(&self, page: u16) -> Result<Vec<u8>, reqwest::Error>; // pdf bytes
}
