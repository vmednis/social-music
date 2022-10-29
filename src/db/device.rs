use crate::db;
use redis::AsyncCommands;

impl db::DbInternal {
    fn key_device(user_id: String) -> String {
        format!("{}:device", user_id)
    }

    pub async fn set_device(&mut self, user_id: String, device_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.set(Self::key_device(user_id), device_id).await.unwrap();
    }

    pub async fn get_device(&mut self, user_id: String) -> Option<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.get(Self::key_device(user_id)).await.ok()
    }
}
