use std::collections::HashMap;

use biliup::credential::LoginInfo;
use reqwest::header::{HeaderValue, COOKIE};
use serde::Deserialize;

use crate::Error;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DanmuInfo {
    pub token: String,
    pub host_list: Vec<DanmuHost>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DanmuHost {
    pub host: String,
    pub port: u16,
    pub wss_port: u16,
    pub ws_port: u16,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PlayInfo {
    pub room_id: u32,
    pub short_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BiliResponse<T> {
    Ok(T),
    Err(i32, String),
}

impl<T> BiliResponse<T> {
    pub fn ok(self) -> Result<T, Error> {
        match self {
            BiliResponse::Ok(data) => Ok(data),
            BiliResponse::Err(code, message) => Err(Error::BiliResponse(code, message)),
        }
    }
}

impl<'de, T> Deserialize<'de> for BiliResponse<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BiliResponseInner<T> {
            code: i32,
            message: String,
            data: Option<T>,
        }

        let inner = BiliResponseInner::deserialize(deserializer)?;
        match inner.data {
            Some(data) => Ok(BiliResponse::Ok(data)),
            None => Ok(BiliResponse::Err(inner.code, inner.message)),
        }
    }
}

pub fn bili_cookies<'a>(login_info: &'a LoginInfo, buvid: &'a str) -> HashMap<&'a str, &'a str> {
    let cookies = match login_info
        .cookie_info
        .get("cookies")
        .and_then(|c| c.as_array())
    {
        Some(c) => c,
        None => return HashMap::new(),
    };

    let mut cookie_map: HashMap<&str, &str> = cookies
        .iter()
        .filter_map(|c| {
            match (
                c.get("name").and_then(|n| n.as_str()),
                c.get("value").and_then(|v| v.as_str()),
            ) {
                (Some(name), Some(value)) => Some((name, value)),
                _ => None,
            }
        })
        .collect();

    cookie_map.insert("buvid3", buvid);
    cookie_map
}

pub fn bili_client(cookies: &HashMap<&str, &str>) -> reqwest::Result<reqwest::Client> {
    let cookie_string = cookies
        .iter()
        .fold(String::new(), |acc, (k, v)| format!("{acc}{k}={v}; "));

    let cookie_value = match HeaderValue::from_str(&cookie_string) {
        Ok(c) => c,
        Err(_) => HeaderValue::from_static(""),
    };

    let mut header_map = reqwest::header::HeaderMap::new();
    header_map.insert(COOKIE, cookie_value);

    reqwest::Client::builder()
        .cookie_store(true)
        .default_headers(header_map)
        .user_agent(USER_AGENT)
        .build()
}

type RoomInfo = (PlayInfo, DanmuInfo);

pub async fn get_room_info(client: &reqwest::Client, room_id: u32) -> Result<RoomInfo, Error> {
    client
        .get(format!("https://live.bilibili.com/{room_id}"))
        .send()
        .await?;

    let play_info = client
        .get("https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo")
        .query(&[
            ("room_id", format!("{room_id}").as_str()),
            ("protocol", "0"),
            ("format", "0"),
            ("codec", "0"),
        ])
        .send()
        .await?
        .json::<BiliResponse<PlayInfo>>()
        .await?
        .ok()?;

    if play_info.room_id != room_id {
        tracing::info!("[{room_id}] real room id: {}", play_info.room_id);
    }

    let danmu_info = client
        .get("https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo")
        .query(&[
            ("id", format!("{}", play_info.room_id).as_str()),
            ("type", "0"),
        ])
        .send()
        .await?
        .json::<BiliResponse<DanmuInfo>>()
        .await?
        .ok()?;

    Ok((play_info, danmu_info))
}
