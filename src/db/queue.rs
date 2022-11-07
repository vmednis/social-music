use redis::AsyncCommands;

use crate::db;

impl db::DbInternal {
    fn key_queue(room_id: String) -> String {
        format!("room:{}:queue", room_id)
    }

    pub async fn push_queue(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.lpush(Self::key_queue(room_id), user_id).await.unwrap();
    }

    pub async fn rem_queue(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .lrem(Self::key_queue(room_id), 0, user_id)
            .await
            .unwrap();
    }

    pub async fn pop_queue(&mut self, room_id: String) -> Option<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.lpop(Self::key_queue(room_id), None).await.unwrap()
    }

    pub async fn list_queue(&mut self, room_id: String) -> Vec<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.lrange(Self::key_queue(room_id), 0, -1).await.unwrap()
    }

    fn key_user_queue(room_id: String, user_id: String) -> String {
        format!("{}:{}", Self::key_queue(room_id), user_id)
    }

    pub async fn push_user_queue(&mut self, room_id: String, user_id: String, track_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .lpush(Self::key_user_queue(room_id, user_id), track_id)
            .await
            .unwrap();
    }

    pub async fn pop_user_queue(&mut self, room_id: String, user_id: String) -> Option<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.lpop(Self::key_user_queue(room_id, user_id), None)
            .await
            .unwrap()
    }
}
