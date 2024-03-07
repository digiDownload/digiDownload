#![feature(try_blocks, async_closure, lazy_cell)]
#![allow(dead_code)]

use crate::digi4school::session;
use crate::error::DigiDownloadError;
use std::env;

mod buffered_response;
mod digi4school;
mod error;
mod scraper;
mod util;

#[tokio::main]
async fn main() {
    try_expect!(
        Result<(), DigiDownloadError>,
        &("Unable to reach the internet. ".to_owned() +
        if cfg!(feature = "route_burp") {
                "Did you start BurpSuite?"
            } else {
                "Check your connection."}
        ),

        {
            let client = session::Session::new(
                env::var("email").unwrap(),
                env::var("password").unwrap()
            ).await?;

            let books = client.get_books().await.unwrap();
            let book = &books[0];

            let scraper = book.get_scraper().await?;

            let mut pdf = scraper.fetch_page_pdf(1).await?;
            pdf.save("/tmp/digi/test.pdf").unwrap();

            let mut full_pdf = scraper.download_book().await?;

            full_pdf.prune_objects();
            full_pdf.compress();

            full_pdf.save("/tmp/digi/full.pdf").unwrap();
        }
    );
}
