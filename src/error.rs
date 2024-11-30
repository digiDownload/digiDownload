use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error(transparent)]
    PdfError(#[from] lopdf::Error),

    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum RedeemCodeError {
    #[error("The provided book code is invalid")]
    BadCode,

    #[error("The provided book code has already been redeemed")]
    ExpiredCode,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Your login information was invalid")]
    BadLogin,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum DigiDownloadError {
    #[error(transparent)]
    Scraper(#[from] ScraperError),

    #[error(transparent)]
    Login(#[from] LoginError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),
}
