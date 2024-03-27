use std::collections::HashMap;

use futures_util::{future, pin_mut, SinkExt, StreamExt};
use tokio_tungstenite::connect_async;

use crate::info::{bili_client, get_danmu_info};
use crate::sub::{auth_sub, heartbeat_sub};
use crate::Error;
use crate::LiveSubHandler;

pub async fn connect_room<H: LiveSubHandler + Sync>(
    cookies: &HashMap<&str, &str>,
    room_id: u32,
    handler: H,
) -> Result<(), Error> {
    let client = bili_client(cookies)?;
    let danmu_info = get_danmu_info(&client, room_id).await?;

    let ws_url = danmu_info
        .host_list
        .first()
        .map(|h| format!("wss://{}:{}/sub", h.host, h.wss_port))
        .ok_or(Error::MissingData("danmu host"))?;

    let (ws_stream, _) = connect_async(ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let writer = async {
        let uid: u64 = cookies
            .get("DedeUserID")
            .ok_or(Error::MissingData("no uid"))?
            .parse()?;

        let buvid = cookies
            .get("buvid3")
            .ok_or(Error::MissingData("no buvid"))?;

        let auth = auth_sub(uid, room_id, buvid, &danmu_info.token)?;
        write.send(auth).await?;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            write.send(heartbeat_sub()).await?;
        }

        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    };

    let reader = async {
        while let Some(message) = read.next().await {
            let data = message?.into_data();
            for reply in crate::sub::decode(data)? {
                handler.handle_reply(reply).await;
            }
        }

        Ok::<(), Error>(())
    };

    pin_mut!(writer, reader);
    future::select(writer, reader).await;

    Ok(())
}
