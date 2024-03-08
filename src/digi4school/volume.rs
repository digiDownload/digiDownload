use crate::buffered_response::BufferedResponse;
use crate::digi4school::book::Book;
use crate::scraper::get_scraper_constructor;
use crate::scraper::scraper_trait::Scraper;
use reqwest::{Client, Url};
use std::sync::{Arc, OnceLock};

pub struct Volume<'a> {
    url: Url,
    resp: OnceLock<BufferedResponse>,

    name: String,
    thumbnail: Url,

    book: &'a Book,
    client: Arc<Client>,
}

impl<'a> Volume<'a> {
    pub(crate) fn new(
        url: Url,
        name: &str,
        thumbnail: Url,
        book: &'a Book,
        client: Arc<Client>,
    ) -> Self {
        Self {
            url,
            resp: OnceLock::default(),

            name: name.to_string(),
            thumbnail,

            book,
            client,
        }
    }

    pub(crate) fn get_from_single_volume_book(book: &Book, resp: BufferedResponse) -> Volume {
        Volume {
            url: resp.url().clone(),
            resp: OnceLock::from(resp),

            name: book.get_name().to_string(),
            thumbnail: book.get_thumbnail(),

            book,
            client: book.get_client(),
        }
    }

    pub async fn get_scraper(&self) -> Result<Box<dyn Scraper + '_>, reqwest::Error> {
        self.gen_response().await?;

        Ok(get_scraper_constructor(&self.url)(
            self.book,
            self.client.clone(),
            self.get_response().await?,
        ))
    }

    async fn get_response(&self) -> Result<&BufferedResponse, reqwest::Error> {
        match self.resp.get() {
            Some(resp) => Ok(resp),
            None => {
                self.gen_response().await?;
                Ok(self.resp.get().unwrap())
            }
        }
    }

    async fn gen_response(&self) -> Result<(), reqwest::Error> {
        if self.resp.get().is_some() {
            return Ok(());
        }

        self.resp
            .set(BufferedResponse::new(self.client.get(self.url.clone()).send().await?).await?)
            .unwrap(); // TODO ???

        Ok(())
    }
}
