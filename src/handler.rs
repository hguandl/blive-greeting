use std::future::Future;

use serde::Deserialize;
use serde_json::Value;
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
    Danmu(DanmuMessage),
    #[serde(rename = "INTERACT_WORD")]
    Interact,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DanmuMessage {
    pub content: String,
    pub uid: u64,
    pub uname: String,
}

impl<'de> Deserialize<'de> for DanmuMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let root: Value = Deserialize::deserialize(deserializer)?;

        let info = root["info"]
            .as_array()
            .ok_or(serde::de::Error::custom("cannot parse array `info`"))?;

        let content = info[1]
            .as_str()
            .ok_or(serde::de::Error::custom("cannot parse str `info[1]`"))?
            .to_string();

        let user = info[2]
            .as_array()
            .ok_or(serde::de::Error::custom("cannot parse array `info[2]`"))?;

        let uid = user[0]
            .as_u64()
            .ok_or(serde::de::Error::custom("cannot parse u64 `info[2][0]`"))?;

        let uname = user[1]
            .as_str()
            .ok_or(serde::de::Error::custom("cannot parse str `info[2][1]`"))?
            .to_string();

        Ok(Self {
            content,
            uid,
            uname,
        })
    }
}
