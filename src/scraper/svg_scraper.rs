use crate::buffered_response::BufferedResponse;
use crate::scraper::base_scraper::BaseScraper;
use crate::scraper::scraper_trait::Scraper;
use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use regex::Regex;
use reqwest::RequestBuilder;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;

#[async_trait]
pub trait SvgScraper: BaseScraper + Sync + Send + Debug {
    /// get an unmodified svg directly from the page
    async fn get_page_raw_svg(&self, page: u16) -> Result<String, reqwest::Error>;
    fn get_image_request(&self, relative_url: &str) -> RequestBuilder;

    async fn get_page_svg(&self, page: u16) -> Result<String, reqwest::Error> {
        let raw_svg = self.get_page_raw_svg(page).await?;
        let mut svg = raw_svg.clone();

        let mut images = HashMap::new();

        let url_regex = Regex::new(&format!("xlink:href=\"({}/.+?)\"/>", page)).unwrap();
        for capture in url_regex.captures_iter(&raw_svg) {
            // TODO add rayon iter??
            let url = capture.get(1).unwrap().as_str();

            // skip already downloaded images TODO: prevent from spawning 2 threads that then both save this ugh
            if images.contains_key(url) {
                continue;
            };

            let resp = BufferedResponse::new(self.get_image_request(url).send().await?).await?;
            images.insert(url, resp);
        }

        for (url, resp) in images {
            let content_type = {
                let content_type_header = resp.headers().get("Content-Type").unwrap_or_else(|| {
                    panic!(
                        "No Content-Type specified on downloaded content: {}",
                        resp.url().as_str()
                    )
                });

                content_type_header
                    .to_str()
                    .unwrap_or_else(|_| {
                        panic!(
                            "Content-Type not according to http spec: {:?}",
                            content_type_header
                        )
                    })
                    .to_owned()
            };

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
        fs::write("/tmp/digi/test.svg", &svg).unwrap(); // TODO remove
        Ok(svg2pdf::convert_str(&svg, svg2pdf::Options::default()).expect("malformed svg found"))
    }
}

#[async_trait]
impl<T> Scraper for T
where
    T: SvgScraper,
{
    async fn fetch_page_pdf(&self, page: u16) -> Result<Vec<u8>, reqwest::Error> {
        SvgScraper::fetch_page_pdf(self, page).await
    }
}
