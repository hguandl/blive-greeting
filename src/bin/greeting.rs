use biliup::credential::login_by_cookies;
use blive_greeting::danmu::send_greeting;
use blive_greeting::gen_buvid3;
use blive_greeting::info::bili_cookies;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let bili = login_by_cookies("cookies.json")
        .await
        .expect("failed to login");

    let rooms = tokio::fs::read_to_string("rooms.txt")
        .await
        .expect("failed to read rooms.txt");

    for room_id in rooms.lines() {
        let room_id = room_id.parse().expect("invalid room id");
        run(&bili, room_id).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn run(bili: &biliup::bilibili::BiliBili, room_id: u32) {
    let buvid = gen_buvid3();
    let cookies = bili_cookies(&bili.login_info, &buvid);

    match send_greeting(&cookies, room_id).await {
        Ok(_) => info!("[{}] greeting sent", room_id),
        Err(e) => error!("[{}] send greeting error: {e}", room_id),
    }
}
