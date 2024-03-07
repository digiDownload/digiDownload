use thiserror::Error;

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
    Login(#[from] LoginError),

    #[error(transparent)]
    Request(#[from] reqwest::Error),
}
