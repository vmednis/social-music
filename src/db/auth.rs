use crate::db;
use redis::AsyncCommands;

impl db::DbInternal {
    fn key_auth(user_id: String) -> String {
        format!("{}:auth", user_id)
    }

    pub async fn set_auth(&mut self, user_id: String, token: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.set(Self::key_auth(user_id), token).await.unwrap();
    }

    pub async fn get_auth(&mut self, user_id: String) -> Option<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.get(Self::key_auth(user_id)).await.ok()
    }
}
