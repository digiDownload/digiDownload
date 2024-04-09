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
            let book = &books[1];

            for (i, book) in books.iter().enumerate() {
                println!("Book {i}: {}", book.title())
            };

            let volume = book.get_volumes().await?;
            let scraper = volume[0].get_scraper().await?;

            let mut full_pdf = scraper.download_book().await?;

            full_pdf.save("/tmp/digi/full.pdf").unwrap();
        }
    );
}
