use crate::error::ScraperError;
use crate::scraper::base_scraper::BaseScraper;
use async_trait::async_trait;
use lopdf::Document;
use std::fmt::Debug;
use std::io::Cursor;

#[async_trait]
pub trait Scraper: BaseScraper + Sync + Send + Debug {
    async fn fetch_page_raw_pdf(&self, page: u16) -> Result<Vec<u8>, reqwest::Error>; // pdf bytes

    async fn fetch_page_pdf(&self, page: u16) -> Result<Document, ScraperError> {
        Ok(Document::load_from(Cursor::new(
            self.fetch_page_raw_pdf(page).await?,
        ))?)
    }
}
