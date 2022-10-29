use crate::db;
use redis::AsyncCommands;

impl db::DbInternal {
    fn key_presence(room_id: String, user_id: String) -> String {
        format!("room:{}:presence:{}", room_id, user_id)
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
}
