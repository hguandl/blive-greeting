use std::{collections::HashMap, time::SystemTime};

use anyhow::{anyhow, Result};

use blive_greeting::info::bili_client;

pub async fn send_greeting(cookies: &HashMap<&str, &str>, room_id: u32) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    let bili_jct = cookies.get("bili_jct").ok_or(anyhow!("no bili_jct"))?;

    let client = bili_client(cookies)?;

    let form = reqwest::multipart::Form::new()
        .text("bubble", "0")
        .text("msg", greeting_word(timestamp))
        .text("color", "5816798")
        .text("mode", "1")
        .text("room_type", "0")
        .text("jumpfrom", "0")
        .text("reply_mid", "0")
        .text("reply_attr", "0")
        .text("replay_dmid", "")
        .text("fontsize", "25")
        .text("rnd", timestamp.to_string())
        .text("roomid", room_id.to_string())
        .text("csrf", bili_jct.to_string())
        .text("csrf_token", bili_jct.to_string());

    client
        .post("https://api.live.bilibili.com/msg/send")
        .header("Referer", format!("https://live.bilibili.com/{room_id}"))
        .multipart(form)
        .send()
        .await?;

    Ok(())
}

fn greeting_word(timestamp: u64) -> &'static str {
    match (timestamp + 8 * 3600) % 86400 {
        00000..=14400 => "晚上好", // 0:00 - 4:00
        14401..=32400 => "早上好", // 4:00 - 9:00
        32401..=41400 => "上午好", // 9:00 - 11:30
        41401..=48600 => "中午好", // 11:30 - 13:30
        48601..=61200 => "下午好", // 13:30 - 17:00
        61201..=86399 => "晚上好", // 17:00 - 24:00
        _ => unreachable!(),
    }
}
