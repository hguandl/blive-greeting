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
                    #[cfg(feature = "debug-danmu")]
                    {
                        let v = serde_json::from_slice::<Value>(&data)?;
                        println!("{}", serde_json::to_string(&v)?);
                    }
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
    pub medal: Option<FanMedal>,
    pub ts: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FanMedal {
    pub level: u64,
    pub name: String,
    pub target_name: String,
    pub room_id: u64,
    pub target_id: u64,
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

        let uid = info[2][0]
            .as_u64()
            .ok_or(serde::de::Error::custom("cannot parse u64 `info[2][0]`"))?;

        let uname = info[2][1]
            .as_str()
            .ok_or(serde::de::Error::custom("cannot parse str `info[2][1]`"))?
            .to_string();

        let medal: Option<FanMedal> = Deserialize::deserialize(&info[3])
            .map_err(|e| serde::de::Error::custom(format!("info[3]: {}", e)))?;

        let ts = info[9]["ts"]
            .as_u64()
            .ok_or(serde::de::Error::custom("cannot parse u64 `info[9][1]`"))?;

        Ok(Self {
            content,
            uid,
            uname,
            medal,
            ts,
        })
    }
}

impl<'de> Deserialize<'de> for FanMedal {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let root: Value = Deserialize::deserialize(deserializer)?;

        let level = root[0]
            .as_u64()
            .ok_or(serde::de::Error::custom("cannot parse u64 `[0]`"))?;

        let name = root[1]
            .as_str()
            .ok_or(serde::de::Error::custom("cannot parse str `[1]`"))?
            .to_string();

        let target_name = root[2]
            .as_str()
            .ok_or(serde::de::Error::custom("cannot parse str `[2]`"))?
            .to_string();

        let room_id = root[3]
            .as_u64()
            .ok_or(serde::de::Error::custom("cannot parse u64 `[3]`"))?;

        let target_id = root[12]
            .as_u64()
            .ok_or(serde::de::Error::custom("cannot parse u64 `[12]`"))?;

        Ok(Self {
            level,
            name,
            target_name,
            room_id,
            target_id,
        })
    }
}
