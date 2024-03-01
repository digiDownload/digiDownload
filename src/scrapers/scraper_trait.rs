use crate::buffered_response::BufferedResponse;
use crate::digi4school::book::Book;
use async_trait::async_trait;
use reqwest::Client;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait Scraper: Sync + Send + Debug {
    fn new_scraper(
        book: &Book,
        client: Arc<Client>,
        resp: BufferedResponse,
    ) -> Box<dyn Scraper + '_>
    where
        Self: Sized;

    async fn fetch_page_count(&self) -> Result<u16, reqwest::Error>;

    async fn fetch_page_pdf(&self, page: u16) -> Result<Vec<u8>, reqwest::Error>; // pdf bytes
}
