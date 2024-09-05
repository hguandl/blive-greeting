use reqwest::{Client, Error};
use serde_json::json;

pub struct OneBot<'a> {
    endpoint: &'a str,
    token: &'a str,
}

pub enum Peer {
    Friend(i64),
    Group(i64),
}

impl<'a> OneBot<'a> {
    pub fn new(endpoint: &'a str, token: &'a str) -> Self {
        Self { endpoint, token }
    }

    pub async fn send_message(&self, peer: &Peer, message: &str) -> Result<(), Error> {
        let (id_kind, id) = match peer {
            Peer::Friend(id) => ("user_id", id),
            Peer::Group(id) => ("group_id", id),
        };

        Client::new()
            .post(format!("{}/send_msg", self.endpoint))
            .bearer_auth(self.token)
            .json(&json!({
                id_kind: id,
                "message": message
            }))
            .send()
            .await?;

        Ok(())
    }
}
