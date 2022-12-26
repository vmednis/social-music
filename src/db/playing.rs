use std::collections::HashMap;

use redis::AsyncCommands;

use crate::db;

impl db::DbInternal {
    fn key_playing(room_id: String) -> String {
        format!("room:{}:playing", room_id)
    }

    pub async fn set_playing(
        &mut self,
        room_id: String,
        track_id: String,
        start_time: u128,
        length: u64,
    ) {
        let mut data = Vec::new();
        data.push(("track_id".to_string(), track_id));
        data.push(("start_time".to_string(), start_time.to_string()));
        data.push(("length".to_string(), length.to_string()));

        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .hset_multiple(Self::key_playing(room_id), &data[..])
            .await
            .unwrap();
    }

    pub async fn get_playing(&mut self, room_id: String) -> Option<Playing> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let data: HashMap<String, String> =
            con.hgetall(Self::key_playing(room_id)).await.unwrap();

        if !data.is_empty() {
            let track_id = data.get("track_id").unwrap().clone();
            let start_time = data.get("start_time").unwrap().parse::<u128>().unwrap();
            let length = data.get("length").unwrap().parse::<u64>().unwrap();

            Some(Playing {
                track_id,
                start_time,
                length,
            })
        } else {
            None
        }
    }
}

pub struct Playing {
    pub track_id: String,
    pub start_time: u128,
    pub length: u64,
}
