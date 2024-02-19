use std::{collections::HashMap, time::SystemTime};

use serde::Deserialize;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use tracing::{debug, error, info};

use crate::{danmu::send_greeting, sub::SubReply};

pub struct LiveSubHandler<'a> {
    room_id: u32,
    cookies: &'a HashMap<&'a str, &'a str>,
    receiver: UnboundedReceiver<SubReply>,
    last_greeting: Mutex<SystemTime>,
}

impl<'a> LiveSubHandler<'a> {
    pub fn new(
        room_id: u32,
        cookies: &'a HashMap<&str, &str>,
        receiver: UnboundedReceiver<SubReply>,
    ) -> Self {
        Self {
            room_id,
            cookies,
            receiver,
            last_greeting: Mutex::new(SystemTime::now()),
        }
    }
}

impl<'a> LiveSubHandler<'a> {
    pub async fn run(&mut self) {
        while let Some(reply) = self.receiver.recv().await {
            match reply {
                SubReply::Heartbeat(data) => {
                    if data == [0, 0, 0, 1].as_slice() {
                        debug!("[{}] heartbeat OK", self.room_id);
                    } else {
                        error!("[{}] heartbeat error", self.room_id);
                    }
                }
                SubReply::Message(data) => match serde_json::from_slice::<LiveMessage>(&data) {
                    Ok(m) => self.handle_message(&m).await,
                    Err(e) => error!("[{}] parse message error: {e}", self.room_id),
                },
                SubReply::Auth(data) => {
                    if data == r#"{"code":0}"# {
                        info!("[{}] auth OK", self.room_id);
                    } else {
                        error!("[{}] auth error", self.room_id);
                        break;
                    }
                }
            }
        }
    }

    async fn handle_message(&self, message: &LiveMessage) {
        match message {
            LiveMessage::Live => {
                let duration = {
                    let mut last = self.last_greeting.lock().await;
                    let now = SystemTime::now();
                    let duration = now.duration_since(*last).unwrap().as_secs();
                    *last = now;
                    duration
                };

                if duration < 10 {
                    debug!("[{}] debounce greeting within {duration}s", self.room_id);
                    return;
                }

                match send_greeting(self.cookies, self.room_id).await {
                    Ok(_) => info!("[{}] greeting sent", self.room_id),
                    Err(e) => error!("[{}] send greeting error: {e}", self.room_id),
                }
            }
            _ => debug!("[{}] received {message:?}", self.room_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "cmd")]
enum LiveMessage {
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
