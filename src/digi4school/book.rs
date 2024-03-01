use crate::buffered_response::BufferedResponse;
use crate::digi4school::lti_form::LTIForm;
use crate::scrapers::get_scraper_new_fn;
use crate::scrapers::scraper_trait::Scraper;
use reqwest::{Client, Response};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Book {
    long_id: String,
    short_id: u16,

    /// Keeps track of the date at which the book was redeemed.
    /// Used to categorize the books in the user interface.
    year: u16,
    name: String,

    client: Arc<Client>,
}

// TODO refactor methods a bit and maybe replace getters with that one crate
impl Book {
    pub fn new(
        long_id: &str,
        short_id: u16,
        expiration_year: u16,
        name: &str,
        client: Arc<Client>,
    ) -> Self {
        Self {
            long_id: long_id.to_string(),
            short_id,

            year: Self::get_year(expiration_year),
            name: name.to_string(),

            client,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_short_id(&self) -> u16 {
        self.short_id
    }

    const fn get_year(expiration_year: u16) -> u16 {
        // Currently all Digi4School
        expiration_year - 6
    }

    pub async fn get_scraper(&self) -> Result<Box<dyn Scraper + '_>, reqwest::Error> {
        // TODO expect number of redirects and parse LTI-form there to obtain url immediately (saves 1 redirect)
        let resp = self
            .follow_lti_form(
                self.client
                    .get(format!("https://digi4school.at/ebook/{}", self.long_id))
                    .send()
                    .await?,
            )
            .await?;

        Ok(get_scraper_new_fn(resp.url())(
            self,
            self.client.clone(),
            resp,
        ))
    }

    async fn follow_lti_form(&self, resp: Response) -> Result<BufferedResponse, reqwest::Error> {
        let resp = BufferedResponse::new(resp).await?;

        Ok(match LTIForm::new(&resp) {
            Some(form) => form.follow_recursively(&self.client).await?,
            None => resp,
        })
    }
}
