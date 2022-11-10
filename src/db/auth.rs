use crate::db;
use redis::AsyncCommands;
use std::collections::HashMap;

impl db::DbInternal {
    fn key_auth(user_id: String) -> String {
        format!("{}:auth", user_id)
    }

    pub async fn set_auth(&mut self, user_id: String, access_token: String, refresh_token: String) {
        let mut data = Vec::new();
        data.push(("access_token".to_string(), access_token));
        data.push(("refresh_token".to_string(), refresh_token));

        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .hset_multiple(Self::key_auth(user_id), &data[..])
            .await
            .unwrap();
    }

    pub async fn get_auth(&mut self, user_id: String) -> Option<Auth> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let data: Option<HashMap<String, String>> =
            con.hgetall(Self::key_auth(user_id)).await.unwrap();

        match data {
            Some(data) => {
                let access_token = data.get("access_token").unwrap().clone();
                let refresh_token = data.get("refresh_token").unwrap().clone();

                Some(Auth {
                    access_token,
                    refresh_token,
                })
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Auth {
    pub access_token: String,
    pub refresh_token: String,
}
