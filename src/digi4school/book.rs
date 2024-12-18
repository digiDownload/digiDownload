use crate::buffered_response::BufferedResponse;
use crate::digi4school::lti_form::LTIForm;
use crate::digi4school::volume::Volume;
use crate::regex_builder;
use getset::{CopyGetters, Getters};
use regex::RegexBuilder;
use reqwest::{Client, Url};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Getters, CopyGetters)]
pub struct Book {
    #[getset(get_copy = "pub(crate)")]
    id: u16,

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
    const BASE_URL: &'static str = "https://a.digi4school.at";

    pub(crate) fn new(
        id: u16,
        expiration_year: u16,
        thumbnail: Url,
        name: &str,
        client: Arc<Client>,
    ) -> Self {
        Self {
            id,

            year: Self::get_redemption_year(expiration_year),
            thumbnail,
            title: name.to_string(),

            client,
        }
    }

    pub async fn get_volumes(&self) -> Result<Vec<Volume>, reqwest::Error> {
        let resp = LTIForm::follow(
            BufferedResponse::new(self.client.get(self.base_url()).send().await?).await?,
            &self.client,
        )
        .await?;

        // If the book is loaded directly (meaning it only has one volume) it will always have a `<DOCTYPE html>` tag.
        if resp.text().starts_with("<!DOCTYPE html>") {
            let volume = Volume::from_single_volume_book(self, resp);
            Ok(vec![volume])
        } else {
            let volumes: Vec<Volume> = regex_builder!(
                    RegexBuilder::new(
                        r#"<a( class="")? href="(.+?)" target="_blank">[\s\S]+?<img src="(.+?)" />[\s\S]+?<div class="tx"><h1>(.+?)</h1></div>"#
                    ).dot_matches_new_line(true)
                ).captures_iter(&resp.text())
                .map(|c| {
                    Volume::new(
                        self.relative_url(c.get(2).unwrap().as_str()), // URL
                        c.get(4).unwrap().into(), // name
                        self.relative_url(c.get(3).unwrap().as_str()), // thumbnail

                        self.client.clone(),
                    )
                })
                .collect();

            assert!(!volumes.is_empty());
            Ok(volumes)
        }
    }

    // Needed for `Volume::from_single_volume_book`
    pub(crate) fn client(&self) -> Arc<Client> {
        self.client.clone()
    }

    const fn get_redemption_year(expiration_year: u16) -> u16 {
        // For some reason books always expire after 6 years.
        expiration_year - 6
    }

    fn relative_url(&self, path: &str) -> Url {
        Url::from_str(&format!("{}/{}", self.base_url(), path)).unwrap()
    }

    fn base_url(&self) -> String {
        format!("{}/ebook/{}", Self::BASE_URL, self.id)
    }
}

impl Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}) {}", self.year(), self.title()))
    }
}
