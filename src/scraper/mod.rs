use crate::buffered_response::BufferedResponse;
use crate::scraper::base_scraper::BaseScraper;
use crate::scraper::scraper_structs::digi4school::Digi4SchoolScraper;
use crate::scraper::scraper_trait::Scraper;
use crate::try_expect;
use reqwest::{Client, Url};
use std::sync::Arc;

pub mod scraper_trait;

mod base_scraper;
mod scraper_structs;
mod svg_scraper;
mod util;

pub fn get_scraper_constructor(
    url: &Url,
) -> for<'a> fn(&'a BufferedResponse, Arc<Client>) -> Box<dyn Scraper + 'a> {
    let base_url = try_expect!(
        Option<_>,
        format!("Invalid URL passed: {url}").as_str(),
        format!("{}://{}", url.scheme(), url.domain()?)
    );

    match base_url.as_str() {
        Digi4SchoolScraper::URL => Digi4SchoolScraper::new_scraper,

        _ => unimplemented!("Scraper for '{base_url}' (full URL: '{url}')\n\
        Please open a github issue with the book you tried downloading and with the url in this error message.")
        // TODO add github issue template and insert link to open new 'Scraper not implemented' issue
    }
}
