use std::{collections::HashMap, time::SystemTime};

use blive_greeting::{LiveMessage, LiveSubHandler, Result};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::danmu::send_greeting;

pub struct LiveGreetingBot<'a> {
    room_id: u32,
    cookies: &'a HashMap<&'a str, &'a str>,
    last_greeting: Mutex<SystemTime>,
}

impl<'a> LiveGreetingBot<'a> {
    pub fn new(room_id: u32, cookies: &'a HashMap<&str, &str>) -> Self {
        Self {
            room_id,
            cookies,
            last_greeting: Mutex::new(SystemTime::now()),
        }
    }
}

impl<'a> LiveSubHandler for LiveGreetingBot<'a> {
    fn get_room_id(&self) -> u32 {
        self.room_id
    }

    async fn handle_message(&self, message: &LiveMessage) -> Result<()> {
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
                    return Ok(());
                }

                match send_greeting(self.cookies, self.room_id).await {
                    Ok(_) => info!("[{}] greeting sent", self.room_id),
                    Err(e) => error!("[{}] send greeting error: {e}", self.room_id),
                }

                Ok(())
            }
            _ => {
                debug!("[{}] received {message:?}", self.room_id);
                Ok(())
            }
        }
    }
}
