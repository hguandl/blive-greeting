use std::collections::HashMap;

use anyhow::{anyhow, Result};
use futures_util::{future, pin_mut, SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tracing::error;

use crate::handler::LiveSubHandler;
use crate::info::{bili_client, get_danmu_info};
use crate::sub::{auth_sub, heartbeat_sub};

pub async fn connect_room(cookies: &HashMap<&str, &str>, room_id: u32) -> Result<()> {
    let client = bili_client(cookies)?;
    let danmu_info = get_danmu_info(&client, room_id).await?;

    let ws_url = danmu_info
        .host_list
        .first()
        .map(|h| format!("wss://{}:{}/sub", h.host, h.wss_port))
        .ok_or(anyhow!("no danmu host found"))?;

    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, read) = ws_stream.split();

    let writer = async {
        let uid: u64 = cookies
            .get("DedeUserID")
            .ok_or(anyhow!("no uid"))?
            .parse()?;

        let auth = auth_sub(uid, room_id, &danmu_info.token)?;
        write.send(auth).await?;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            write.send(heartbeat_sub()).await?;
        }

        #[allow(unreachable_code)]
        anyhow::Ok(())
    };

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let reader = read.for_each(|message| async {
        match message {
            Ok(message) => {
                let data = message.into_data();
                if let Err(e) = crate::sub::decode_vec(data, &tx) {
                    error!("[{room_id}] decode: {e}");
                }
            }
            Err(e) => error!("[{room_id}] read: {e}"),
        }
    });

    let mut sub_handler = LiveSubHandler::new(room_id, cookies, rx);
    let handler = sub_handler.run();

    pin_mut!(writer, reader, handler);
    let trigger = future::select(writer, reader);
    future::select(trigger, handler).await;

    Ok(())
}
