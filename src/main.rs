mod greeting_bot;

use biliup::credential::login_by_cookies;
use blive_greeting::info::bili_cookies;
use blive_greeting::{connect_room, gen_buvid3};
use greeting_bot::LiveGreetingBot;
use tracing::error;

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let bili = login_by_cookies("cookies.json")
        .await
        .expect("failed to login");

    tokio::join!(run(&bili, 4588774), run(&bili, 21669627));
}

async fn run(bili: &biliup::bilibili::BiliBili, room_id: u32) {
    loop {
        let buvid = gen_buvid3();
        let cookies = bili_cookies(&bili.login_info, &buvid);
        let handler = LiveGreetingBot::new(room_id, &cookies);
        match connect_room(&cookies, room_id, handler).await {
            Ok(_) => (),
            Err(e) => error!("failed to connect room {room_id}: {e}"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
