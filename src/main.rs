mod danmu;
mod handler;
mod info;
mod live;
mod sub;

use biliup::credential::login_by_cookies;

use live::connect_room;

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let bili = login_by_cookies("cookies.json")
        .await
        .expect("failed to login");

    let results = tokio::join!(connect_room(&bili, 4588774), connect_room(&bili, 21669627));

    eprintln!("{:?}", results);
}
