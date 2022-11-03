use crate::db;
use futures_util::StreamExt;
use redis::AsyncCommands;

impl db::DbInternal {
    fn key_presence(room_id: String, user_id: String) -> String {
        format!("room:{}:presence:{}", room_id, user_id)
    }

    fn rkey_presence(key: String) -> String {
        key.split(":presence:").last().unwrap().to_string()
    }

    pub async fn add_presence(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .set(Self::key_presence(room_id.clone(), user_id.clone()), "")
            .await
            .unwrap();
        self.keep_alive_presence(room_id, user_id).await;
    }

    pub async fn remove_presence(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.del(Self::key_presence(room_id, user_id)).await.unwrap();
    }

    pub async fn keep_alive_presence(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .expire(Self::key_presence(room_id, user_id), 5)
            .await
            .unwrap();
    }

    pub async fn scan_presence(&mut self, room_id: String) -> Vec<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let iter = con.scan_match(Self::key_presence(room_id, "*".to_string())).await.unwrap();
        iter.map(|key| Self::rkey_presence(key)).collect().await
    }

    fn key_presences(room_id: String) -> String {
        format!("room:{}:presences", room_id)
    }

    pub async fn del_presences(&mut self, room_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.del(Self::key_presences(room_id)).await.unwrap();
    }

    pub async fn add_presences(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.sadd(Self::key_presences(room_id), user_id).await.unwrap();
    }

    pub async fn rem_presences(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.srem(Self::key_presences(room_id), user_id).await.unwrap();
    }

    pub async fn list_presences(&mut self, room_id: String) -> Vec<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let iter = con.sscan(Self::key_presences(room_id)).await.unwrap();
        iter.collect().await
    }

    fn key_presence_keyspace(room_id: String) -> String {
        format!("__keyspace*__:{}", Self::key_presence(room_id, "*".to_string()))
    }

    pub async fn subscribe_presence(&mut self, room_id: String) -> tokio::sync::mpsc::Receiver<PresenceEvent>{
        let (tx, rx) = tokio::sync::mpsc::channel(5);

        let con = self.blockable_client().get_async_connection().await.unwrap();

        tokio::task::spawn(async move {
            let mut pubsub = con.into_pubsub();
            pubsub.psubscribe(Self::key_presence_keyspace(room_id)).await.unwrap();
            let mut stream = pubsub.on_message();

            loop {
                let message = stream.next().await.unwrap();
                let activity = match message.get_payload::<String>().unwrap().as_str() {
                    "new" => Some(PresenceEventActivty::Join),
                    "del" => Some(PresenceEventActivty::Leave),
                    "expired" => Some(PresenceEventActivty::Leave),
                    _ => None
                };

                if let Some(activity) = activity {
                    let user_id = Self::rkey_presence(message.get_channel_name().to_string());
                    let event = PresenceEvent {
                        user_id,
                        activity
                    };

                    tx.send(event).await.unwrap();
                }
            }
        });

        rx
    }
}

#[derive(Debug)]
pub enum PresenceEventActivty {
    Join,
    Leave
}

#[derive(Debug)]
pub struct PresenceEvent {
    pub user_id: String,
    pub activity: PresenceEventActivty,
}
