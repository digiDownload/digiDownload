use crate::digi4school::book::Book;
use crate::error::{LoginError, RedeemCodeError};
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

#[derive(Serialize)]
struct RedeemCodeData {
    code: String,
    id: u32,
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

    /// Attempt to redeem the provided code (XXXX-XXXX-XXXX), returns whether the action succeeds.
    pub async fn redeem_code(&self, code: String) -> Result<(), RedeemCodeError> {
        let resp_content = self
            .client
            .post(format!("{}/br/xhr/einloesen", Self::BASE_URL))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(RedeemCodeData { code, id: 0 }).unwrap())
            .send()
            .await?
            .text()
            .await?;

        let err_code = regex!(r#""err"\s*:\s*(\d+)"#)
            .captures(resp_content.as_str())
            .and_then(|cap| cap.get(1))
            .and_then(|err_code| err_code.as_str().parse::<u32>().ok())
            .unwrap();

        match err_code {
            0 => Ok(()),
            106 => Err(RedeemCodeError::BadCode),
            100 => Err(RedeemCodeError::ExpiredCode),
            err => todo!("Handle error code: {}", err),
        }
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
