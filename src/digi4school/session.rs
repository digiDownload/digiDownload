use crate::digi4school::book::Book;
use crate::error::LoginError;
use crate::regex;
use reqwest::{Client, Url};
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;

pub struct Session {
    client: Arc<Client>,
}

#[derive(Serialize)]
struct LoginData {
    email: String,
    password: String,
    indefinite: u8,
}

impl Session {
    pub(crate) const BASE_URL: &'static str = "https://digi4school.at";

    pub async fn new(email: String, password: String) -> Result<Self, LoginError> {
        let builder = Client::builder().cookie_store(true);

        #[cfg(feature = "route_burp")]
        let builder = builder
            .add_root_certificate(
                reqwest::Certificate::from_der(include_bytes!("../../CERT.DER")).unwrap(),
            )
            .proxy(reqwest::Proxy::https("127.0.0.1:8080").unwrap());

        let session = Self {
            client: Arc::new(builder.build().unwrap()),
        };
        session.login(email, password, false).await?;

        Ok(session)
    }

    pub async fn get_books(&self) -> Result<Vec<Book>, reqwest::Error> {
        let resp = self
            .client
            .get(format!("{}/ebooks", Self::BASE_URL))
            .send()
            .await?;

        Ok(
            regex!(
                r"data-code='(.+?)' data-id='(\d+?)'.+?<img src='(.+?)'>.+?<h1>(.+?)</h1>.+?bis (\d{1,2}\.\d{1,2})\.(\d+)"
            )
            .captures_iter(&resp.text().await?)
            .inspect(|m| assert_eq!(m.get(5).unwrap().as_str(), "31.10"))
            .map(|m| {
                Book::new(
                    m.get(2).unwrap().as_str().parse().unwrap(), // id
                    m.get(6).unwrap().as_str().parse().unwrap(), // expiration year
                    Url::from_str(m.get(3).unwrap().as_str()).expect("Thumbnail URL is invalid"), // thumbnail
                    m.get(4).unwrap().as_str(), // title

                    self.client.clone(),
                )
            })
            .collect())
    }

    pub fn redeem_code(&self) -> bool {
        todo!("Program once I can test redeeming a code")
    }

    async fn login(
        &self,
        email: String,
        password: String,
        remember_login: bool,
    ) -> Result<(), LoginError> {
        let resp_content = self
            .client
            .post(format!("{}/br/xhr/login", Self::BASE_URL))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(
                serde_urlencoded::to_string(LoginData {
                    email,
                    password,
                    indefinite: u8::from(remember_login),
                })
                .unwrap(),
            )
            .send()
            .await?
            .text()
            .await?;

        match resp_content.as_str() {
            "OK" => Ok(()),
            "KO" => Err(LoginError::BadLogin),
            _ => panic!("Bad login-form response: {}", resp_content),
        }
    }
}
