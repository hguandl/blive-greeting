mod handler;
mod live;
mod sub;

pub mod info;

pub use handler::{LiveMessage, LiveSubHandler};
pub use live::connect_room;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected API response ({0}): {1}")]
    BiliResponse(i32, String),

    #[error("invalid sub: {0}")]
    DecodeSub(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("missing data: {0}")]
    MissingData(&'static str),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
}
