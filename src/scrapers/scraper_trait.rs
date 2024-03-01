use crate::buffered_response::BufferedResponse;
use crate::digi4school::book::Book;
use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use regex::Regex;
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
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

#[async_trait]
pub trait SvgScraper {
    // TODO add documentation
    async fn get_page_raw_svg(&self, page: u16) -> Result<String, reqwest::Error>;
    fn get_image_request(&self, relative_url: &str) -> RequestBuilder;

    async fn get_page_svg(&self, page: u16) -> Result<String, reqwest::Error> {
        let raw_svg = self.get_page_raw_svg(page).await?;
        let mut svg = raw_svg.clone();

        let mut images = HashMap::new();

        let url_regex = Regex::new(&format!("xlink:href=\"({}/.+?)\"/>", page)).unwrap();
        for capture in url_regex.captures_iter(&raw_svg) {
            // TODO add rayon iter??
            let url = capture.get(1).expect("").as_str(); // TODO

            // skip already downloaded images TODO: prevent from spawning 2 threads that then both save this ugh weoigjeiowjgoiwejoiwj
            if images.contains_key(url) {
                continue;
            };

            let resp = BufferedResponse::new(self.get_image_request(url).send().await?).await?;
            images.insert(url, resp);
        }

        for (url, resp) in images {
            println!("replaced {url} with ..."); // TODO rm

            let content_type = resp
                .headers()
                .get("Content-Type")
                .unwrap() // TODO
                .to_str()
                .unwrap() // TODO
                .to_owned();

            svg = svg.replace(
                url,
                &format!(
                    "data:{};base64,{}",
                    content_type,
                    &BASE64_STANDARD.encode(resp.bytes())
                ),
            );
        }

        Ok(svg)
    }

    async fn fetch_page_pdf(&self, page: u16) -> Result<Vec<u8>, reqwest::Error> {
        let svg = self.get_page_svg(page).await?;

        fs::write("/tmp/digi/test.svg", &svg).unwrap(); // TODO

        Ok(svg2pdf::convert_str(&svg, svg2pdf::Options::default()).expect(" ")) // TODO
    }
}
