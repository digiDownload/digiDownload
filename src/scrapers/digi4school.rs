use crate::buffered_response::BufferedResponse;
use crate::digi4school::book::Book;
use crate::regex;
use crate::scrapers::scraper_trait::Scraper;
use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use regex::Regex;
use reqwest::{Client, RequestBuilder};
use std::fs;
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

    async fn get_page_svg(&self, page: u16) -> Result<String, reqwest::Error> {
        assert!(
            page <= self.page_count,
            "tried downloading invalid page: {page}/{}",
            self.page_count
        );

        let url = format!("{}/{page}.svg", self.get_book_url());

        let content = self.client.get(url).send().await?.text().await?;
        let content = self.embed_svg_images(content, page).await?;

        // Ok(content)

        // TODO alarm
        fs::write("/tmp/test.svg", &content).unwrap();

        Ok(content)
    }

    async fn embed_svg_images(&self, svg: String, page: u16) -> Result<String, reqwest::Error> {
        let mut svg = svg.clone();

        let url_regex = Regex::new(&format!("xlink:href=\"({}/.+?)\"/>", page)).unwrap();

        for capture in url_regex.captures_iter(&svg.clone()) {
            let url = capture.get(1).expect("").as_str(); // TODO
            let resp = self.download_image(url).send().await?;

            let content_type = resp
                .headers()
                .get("Content-Type")
                .unwrap() // TODO
                .to_str()
                .unwrap() // TODO
                .to_owned();

            println!("replaced {} with data:{};base64,[...]", url, content_type);

            svg = svg.replace(
                url,
                &format!(
                    "data:{};base64,{}",
                    content_type,
                    BASE64_STANDARD.encode(resp.bytes().await?)
                ),
            );
        }

        Ok(svg)
    }

    fn download_image(&self, relative_url: &str) -> RequestBuilder {
        let url = format!("{}/{}", self.get_book_url(), relative_url);
        self.client.get(url)
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
                .captures(resp.text())
                .unwrap_or_else(panic_closure!())
                .get(1)
                .unwrap_or_else(panic_closure!())
                .as_str(),
        )
        .unwrap()
    }
}

#[async_trait]
impl Scraper for Digi4SchoolScraper<'_> {
    fn new_scraper(
        book: &Book,
        client: Arc<Client>,
        resp: BufferedResponse,
    ) -> Box<dyn '_ + Scraper>
    where
        Self: Sized,
    {
        Box::new(Digi4SchoolScraper {
            // TODO understand why Digi4SchoolScraper is different from Self
            book, // https://users.rust-lang.org/t/fn-pointer-of-constructor-with-explicit-lifetimes/107439
            client,

            page_count: Self::get_page_count(&resp),
        })
    }

    async fn fetch_page_count(&self) -> Result<u16, reqwest::Error> {
        Ok(self.page_count)
    }

    async fn fetch_page_pdf(&self, page: u16) -> Result<Vec<u8>, reqwest::Error> {
        let svg_page = self.get_page_svg(page).await?;

        let svg_page = Regex::new(&format!("xlink:href=.({}/.+?)\"/>", page))
            .unwrap() // TODO expect
            .replace_all(
                &svg_page,
                &format!(
                    "xlink:href=\"{}/ebook/{}/$1\"/>",
                    Self::URL,
                    self.book.get_short_id()
                ),
            );

        fs::write("/tmp/digi/test.svg", &*svg_page).unwrap();

        Ok(svg2pdf::convert_str(&svg_page, svg2pdf::Options::default())
            .expect("received malformed svg"))

        // TODO add digi4school 'bookmarks' as pdf navigation headers
    }
}
