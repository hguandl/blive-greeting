use std::future::Future;

use serde::Deserialize;
use tracing::{debug, error, info};

use crate::sub::SubReply;

pub trait LiveSubHandler {
    fn get_room_id(&self) -> u32;

    fn handle_message(&self, message: &LiveMessage) -> impl Future<Output = ()> + Send;

    fn handle_reply(&self, reply: SubReply) -> impl Future<Output = ()> + Send
    where
        Self: Sync,
    {
        async {
            match reply {
                SubReply::Heartbeat(data) => {
                    if data == [0, 0, 0, 1].as_slice() {
                        debug!("[{}] heartbeat OK", self.get_room_id());
                    } else {
                        error!("[{}] heartbeat error", self.get_room_id());
                    }
                }
                SubReply::Message(data) => match serde_json::from_slice::<LiveMessage>(&data) {
                    Ok(m) => self.handle_message(&m).await,
                    Err(e) => error!("[{}] parse message error: {e}", self.get_room_id()),
                },
                SubReply::Auth(data) => {
                    if data == r#"{"code":0}"# {
                        info!("[{}] auth OK", self.get_room_id());
                    } else {
                        error!("[{}] auth error", self.get_room_id());
                        return;
                    }
                }
            }
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
