use crate::buffered_response::BufferedResponse;
use crate::digi4school::book::Book;
use crate::regex;
use crate::scraper::base_scraper::BaseScraper;
use crate::scraper::scraper_trait::Scraper;
use crate::scraper::svg_scraper::SvgScraper;
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Digi4SchoolScraper<'a> {
    book: &'a Book,
    client: Arc<Client>,

    page_count: u16,
}

impl<'a> Digi4SchoolScraper<'a> {
    pub const URL: &'static str = "https://a.digi4school.at";

    fn get_book_url(&self) -> String {
        format!("{}/ebook/{}", Self::URL, self.book.get_short_id())
    }

    /// Takes first response from the `LTIForm` redirects as an input
    fn get_page_count(resp: &BufferedResponse) -> u16 {
        macro_rules! panic_closure {
            () => {|| panic!(
                "{} didn't behave as expected. Couldn't find the page number\nResponse:\n{}",
                resp.url(),
                resp.text())};
        }

        u16::from_str(
            regex!(r"IDRViewer\.makeNavBar\((\d+),'\.jpg'")
                .captures(&resp.text())
                .unwrap_or_else(panic_closure!())
                .get(1)
                .unwrap_or_else(panic_closure!())
                .as_str(),
        )
        .unwrap()
    }
}

#[async_trait]
impl SvgScraper for Digi4SchoolScraper<'_> {
    async fn get_page_raw_svg(&self, page: u16) -> Result<String, reqwest::Error> {
        assert!(
            page <= self.page_count,
            "tried downloading invalid page: {page}/{}",
            self.page_count
        );

        let url = format!("{}/{page}.svg", self.get_book_url());
        Ok(self.client.get(url).send().await?.text().await?)
    }

    fn get_image_request(&self, relative_url: &str) -> RequestBuilder {
        let url = format!("{}/{}", self.get_book_url(), relative_url);
        self.client.get(url)
    }
}

#[async_trait]
impl BaseScraper for Digi4SchoolScraper<'_> {
    fn new_scraper<'a>(
        book: &'a Book,
        client: Arc<Client>,
        resp: &'a BufferedResponse,
    ) -> Box<dyn 'a + Scraper>
    where
        Self: Sized,
    {
        Box::new(Digi4SchoolScraper {
            // TODO understand why Digi4SchoolScraper is different from Self
            book, // https://users.rust-lang.org/t/fn-pointer-of-constructor-with-explicit-lifetimes/107439
            client,

            page_count: Self::get_page_count(resp),
        })
    }

    async fn fetch_page_count(&self) -> Result<u16, reqwest::Error> {
        Ok(self.page_count)
    }
}
