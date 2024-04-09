use crate::buffered_response::BufferedResponse;
use crate::digi4school::lti_form::LTIForm;
use crate::digi4school::session::Session;
use crate::digi4school::volume::Volume;
use crate::regex;
use getset::{CopyGetters, Getters};
use reqwest::{Client, Url};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct Book {
    long_id: String,
    #[getset(get_copy = "pub(crate)")]
    short_id: u16,

    /// Keeps track of the date at which the book was obtained.
    /// Used to categorize the books in the user interface.
    #[getset(get_copy = "pub")]
    year: u16,
    #[getset(get = "pub")]
    title: String,
    #[getset(get = "pub")]
    thumbnail: Url,

    client: Arc<Client>,
}

impl Book {
    const DOMAIN: &'static str = "a.digi4school.at";
    const BASE_URL: &'static str = "https://a.digi4school.at";

    pub fn new(
        long_id: &str,
        short_id: u16,
        expiration_year: u16,
        thumbnail: Url,
        name: &str,
        client: Arc<Client>,
    ) -> Self {
        Self {
            long_id: long_id.to_string(),
            short_id,

            year: Self::get_redemption_year(expiration_year),
            thumbnail,
            title: name.to_string(),

            client,
        }
    }

    pub(crate) fn client(&self) -> Arc<Client> {
        self.client.clone()
    }

    pub async fn get_volumes(&self) -> Result<Vec<Volume>, reqwest::Error> {
        let resp = LTIForm::follow(
            BufferedResponse::new(
                self.client
                    .get(format!("{}/ebook/{}", Session::BASE_URL, self.long_id))
                    .send()
                    .await?,
            )
            .await?,
            &self.client,
        )
        .await?;

        let text = resp.text();

        // If the book is loaded directly it will always have a `<DOCTYPE html>` tag
        // I can't guarantee that this will also hold true for all possible Scrapers so we check the URL
        if resp.url().domain().unwrap() != Self::DOMAIN || text.starts_with("<!DOCTYPE html>") {
            let volume = Volume::get_from_single_volume_book(self, resp);
            Ok(vec![volume])
        } else {
            // TODO this is cheap af
            let text = text.replace('\n', " "); // get regex to match newlines'

            let volumes: Vec<Volume> = regex!(r#"<a( class="")? href="(.+?)" target="_blank">.+?<img src="(.+?)" />.+?<div class="tx"><h1>(.+?)</h1></div>"#)
                    .captures_iter(&text)
                    .map(|c| {
                        Volume::new(
                            // TODO fix relative URLs ?????? Wtf did I mean
                            Url::from_str(&format!("{}/{}", self.relative_url(), c.get(2).unwrap().as_str())).unwrap(), // URL
                            c.get(3).unwrap().into(), // name
                            Url::from_str(&format!("{}/{}", self.relative_url(), c.get(4).unwrap().as_str())).unwrap(), // thumbnail

                            self.client.clone()
                        )
                    })
                    .collect();

            assert!(!volumes.is_empty());
            Ok(volumes)
        }
    }

    const fn get_redemption_year(expiration_year: u16) -> u16 {
        // Currently all Digi4School
        expiration_year - 6
    }

    fn relative_url(&self) -> String {
        format!("{}/ebook/{}", Self::BASE_URL, self.short_id)
    }
}
