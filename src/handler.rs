use std::future::Future;

use serde::Deserialize;
use tracing::{debug, info};

use crate::{sub::SubReply, Error, Result};

pub trait LiveSubHandler {
    fn get_room_id(&self) -> u32;

    fn handle_message(&self, message: &LiveMessage) -> impl Future<Output = Result<()>> + Send;

    fn handle_reply(&self, reply: SubReply) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sync,
    {
        async {
            match reply {
                SubReply::Heartbeat(data) => {
                    if data == [0, 0, 0, 1].as_slice() {
                        debug!("[{}] heartbeat OK", self.get_room_id());
                    } else {
                        return Err(Error::Handler(self.get_room_id(), "heartbeat"));
                    }
                }
                SubReply::Message(data) => {
                    let m = serde_json::from_slice::<LiveMessage>(&data)?;
                    self.handle_message(&m).await?;
                }
                SubReply::Auth(data) => {
                    if data == r#"{"code":0}"# {
                        info!("[{}] auth OK", self.get_room_id());
                    } else {
                        return Err(Error::Handler(self.get_room_id(), "auth"));
                    }
                }
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "cmd")]
pub enum LiveMessage {
    #[serde(rename = "LIVE")]
    Live,
    #[serde(rename = "PREPARING")]
    Preparing,
    #[serde(rename = "DANMU_MSG")]
    Danmu,
    #[serde(rename = "INTERACT_WORD")]
    Interact,
    #[serde(other)]
    Other,
}
