use reqwest::Response;
use std::ops::Deref;
use std::str::from_utf8;

/// Stores a response and keeps track of its body in one type.
/// Circumvents consumption of Response by `.text()` or `.bytes()`
/// Does not lazily load the body -> Use only use if you are sure you need the text
pub struct BufferedResponse {
    resp: Response,
    buf: String,
}

impl BufferedResponse {
    pub async fn new(mut resp: Response) -> Result<Self, reqwest::Error> {
        let buf = Self::get_buf(&mut resp).await?;

        Ok(Self { resp, buf })
    }

    pub fn text(&self) -> &str {
        &self.buf
    }

    async fn get_buf(resp: &mut Response) -> Result<String, reqwest::Error> {
        let content_length = resp.content_length();

        let mut buf: String;
        match content_length {
            Some(length) => {
                buf = String::with_capacity(length.try_into().unwrap());
            }
            None => buf = String::new(),
        }

        loop {
            if let Some(chunk) = resp.chunk().await? {
                buf += from_utf8(&chunk).expect("expected charset to be utf8");
            } else {
                return Ok(buf);
            };
        }
    }
}

impl Deref for BufferedResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}
