use anyhow::{bail, Result};
use brotli::Decompressor;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::read::GzDecoder;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

use crate::info::BUVID3;

pub enum SubReply {
    Heartbeat(Bytes),
    Message(Bytes),
    Auth(Bytes),
}

pub fn auth_sub(uid: u64, room_id: u32, token: &str) -> Result<Message> {
    let bytes = encode_json(
        json!({
            "uid": uid,
            "roomid": room_id,
            "protover": 3,
            "buvid": BUVID3.trim(),
            "platform": "web",
            "type": 2,
            "key": token,
        }),
        7,
    )?;

    Ok(Message::binary(bytes))
}

pub fn heartbeat_sub() -> Message {
    let bytes = encode_str("[object Object]", 2);
    Message::binary(bytes)
}

fn encode_bytes(data: &[u8], op_code: u32) -> Vec<u8> {
    let size = 16 + data.len();
    let mut buf = BytesMut::with_capacity(size);
    buf.put_u32(size as u32);
    buf.put_u16(16);
    buf.put_u16(1);
    buf.put_u32(op_code);
    buf.put_u32(1);
    buf.put_slice(data);
    buf.to_vec()
}

fn encode_json<T>(data: T, op_code: u32) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let json_string = serde_json::to_string(&data)?;
    Ok(encode_str(&json_string, op_code))
}

fn encode_str(data: &str, op_code: u32) -> Vec<u8> {
    encode_bytes(data.as_bytes(), op_code)
}

pub fn decode(data: Vec<u8>) -> Result<Vec<SubReply>> {
    let mut replies = Vec::new();
    decode_vec(data, &mut replies)?;
    Ok(replies)
}

fn decode_vec(data: Vec<u8>, replies: &mut Vec<SubReply>) -> Result<()> {
    decode_bytes(Bytes::from(data), replies)
}

fn decode_bytes(mut data: Bytes, replies: &mut Vec<SubReply>) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }

    if data.remaining() < 6 {
        bail!("invalid length data");
    }

    let size = data.get_u32() as usize;
    let header_len = data.get_u16() as usize;

    if header_len < 16 || data.remaining() < 10 {
        bail!("incomplete header");
    }

    let kind = data.get_u16();
    let op_code = data.get_u32();
    let _sequence = data.get_u32();

    if data.remaining() < size + 16 - 2 * header_len {
        bail!("incomplete body")
    }

    let body = data.slice(header_len - 16..size - header_len);

    match (kind, op_code) {
        (0, 5) => replies.push(SubReply::Message(body)),
        (_, 3) => return Ok(replies.push(SubReply::Heartbeat(body))),
        (_, 8) => return Ok(replies.push(SubReply::Auth(body))),
        (2, _) => {
            let mut decoder = GzDecoder::new(body.reader());
            let mut output = Vec::new();
            std::io::copy(&mut decoder, &mut output)?;
            return decode_vec(output, replies);
        }
        (3, _) => {
            let mut decompressor = Decompressor::new(body.reader(), 4096);
            let mut output = Vec::new();
            std::io::copy(&mut decompressor, &mut output)?;
            return decode_vec(output, replies);
        }
        _ => return Ok(()),
    }

    data.advance(size - header_len);
    if data.has_remaining() {
        return decode_bytes(data, replies);
    }

    Ok(())
}
