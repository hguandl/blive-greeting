use std::time::{SystemTime, UNIX_EPOCH};

fn gen_hex(len: u32) -> String {
    let mut hex = String::new();
    for _ in 0..len {
        let value = rand::random::<u8>() % 16;
        hex.push_str(&format!("{:X}", value));
    }
    hex
}

fn gen_ts() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let remain = (ts % 10000).to_string();
    let prefix = "0".repeat(5 - remain.len());
    prefix + &remain
}

pub fn gen_buvid3() -> String {
    format!(
        "{}-{}-{}-{}-{}{}infoc",
        gen_hex(8),
        gen_hex(4),
        gen_hex(4),
        gen_hex(4),
        gen_hex(12),
        gen_ts()
    )
}
