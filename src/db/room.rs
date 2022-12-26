use crate::db;
use redis::{AsyncCommands, Commands};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::{HashSet, HashMap};

impl db::DbInternal {
    fn key_room(room_id: String) -> String {
        format!("room:{}", room_id)
    }

    fn key_rooms() -> String {
        format!("rooms")
    }

    fn key_rooms_free() -> String {
        format!("rooms_free")
    }

    fn key_room_claimed(room_id: String) -> String {
        format!("{}:claimed", Self::key_room(room_id))
    }

    pub async fn create_room(&mut self, room: Room) -> Result<(), Vec<String>> {
        match room.validate() {
            Ok(_) => {
                let room_id = room.id.clone();
                let key = Self::key_room(room_id.clone());
                let args: Vec<(String, String)> = room.into();

                //Have to block so that no one intrudes the transaction unfortunately so have to keep it short
                let mut con = self.client.get_connection().unwrap();
                let _: () = redis::cmd("WATCH")
                    .arg(key.clone())
                    .query(&mut con)
                    .unwrap();
                let exists: i32 = con.exists(key.clone()).unwrap();

                let _: () = redis::cmd("MULTI").query(&mut con).unwrap();

                if exists == 0 {
                    let _: () = con.hset_multiple(key.clone(), &args[..]).unwrap();
                }

                let res: Vec<()> = redis::cmd("EXEC").query(&mut con).unwrap();

                if res.is_empty() {
                    let mut errors = Vec::new();
                    errors.push(format!("Room id {} is already taken.", room_id.clone()));
                    Err(errors)
                } else {
                    let _: () = con.sadd(Self::key_rooms(), room_id).unwrap();
                    Ok(())
                }
            }
            Err(errors) => Err(errors),
        }
    }

    pub async fn exists_room(&mut self, room_id: String) -> bool {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.exists(Self::key_room(room_id)).await.unwrap()
    }

    pub async fn get_room(&mut self, room_id: String) -> Option<Room> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let data: Option<HashMap<String, String>> = con.hgetall(Self::key_room(room_id)).await.unwrap();
        match Room::try_from(data) {
            Ok(data) => Some(data),
            Err(e) => {
                log::debug!("Failed to get room with '{}'", e);
                None
            }
        }
    }

    pub async fn list_rooms(&mut self) -> Vec<Room> {
        let mut con = self.client.get_async_connection().await.unwrap();
        let room_ids: Vec<String> = con.smembers(Self::key_rooms()).await.unwrap();
        std::mem::drop(con);

        let mut rooms = Vec::new();
        for room_id in room_ids {
            //Could be kinda slow, maybe rethink this?
            rooms.push(self.get_room(room_id).await.unwrap());
        }

        rooms
    }

    pub async fn offer_room(&mut self, room_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.lpush(Self::key_rooms_free(), room_id).await.unwrap();
    }

    pub async fn claim_room(&mut self) -> tokio::sync::oneshot::Receiver<Option<String>> {
        let client = self.blockable_client();
        let mut con = client.get_async_connection().await.unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::task::spawn(async move {
            let res: Vec<String> = con.blpop(Self::key_rooms_free(), 5).await.unwrap();

            let res = if res.is_empty() {
                //No rooms to be claimed
                None
            } else {
                let room_id = res.get(1).unwrap().clone();
                let key = Self::key_room_claimed(room_id.clone());

                //Have to block, so try to be as fast as possible
                let mut inner_con = client.get_connection().unwrap();
                let _: () = redis::cmd("WATCH")
                    .arg(key.clone())
                    .query(&mut inner_con)
                    .unwrap();
                let exists: i32 = inner_con.exists(key.clone()).unwrap();

                let _: () = redis::cmd("MULTI").query(&mut inner_con).unwrap();

                if exists == 0 {
                    let _: () = inner_con.set(key.clone(), "").unwrap();
                }

                let res: Vec<()> = redis::cmd("EXEC").query(&mut inner_con).unwrap();

                if res.is_empty() {
                    //Room already claimed
                    None
                } else {
                    let _: () = con.expire(key.clone(), 5).await.unwrap();
                    Some(room_id.clone())
                }
            };

            tx.send(res).unwrap();
        });

        rx
    }

    pub async fn keep_alive_room_claim(&mut self, room_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .expire(Self::key_room_claimed(room_id), 5)
            .await
            .unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub title: String,
    pub owner: String,
}

impl Room {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let protected_ids = HashSet::from(["new"]);

        if self.id.len() == 0 {
            errors.push("Room id must not be empty.".to_string());
        }

        if self.id.len() >= 32 {
            errors.push("Room id must be less than 32 characters long.".to_string());
        }

        if protected_ids.contains(&self.id[..]) {
            errors.push(format!("Room id can't be {}.", self.id.clone()));
        }

        let allowed_chars = Regex::new(r"^[a-zA-Z\d-]*$").unwrap();
        if !allowed_chars.is_match(&self.id[..]) {
            errors.push("Room id contains forbidden character.".to_string());
        }

        if self.title.len() == 0 {
            errors.push("Room name can't be empty".to_string());
        }

        if self.title.len() >= 64 {
            errors.push("Room title must be less than 64 characters long.".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Into<Vec<(String, String)>> for Room {
    fn into(self) -> Vec<(String, String)> {
        let mut args: Vec<(String, String)> = Vec::new();
        args.push(("id".to_string(), self.id));
        args.push(("title".to_string(), self.title));
        args.push(("owner".to_string(), self.owner));

        args
    }
}


impl TryFrom<Option<HashMap<String, String>>> for Room {
    type Error = &'static str;

    fn try_from(data: Option<HashMap<String, String>>) -> Result<Self, Self::Error> {
        let data = data.ok_or("Didn't find room with this id")?;

        let id = data.get("id").ok_or("Missing id field")?.clone();
        let title = data.get("title").ok_or("Missing title field")?.clone();
        let owner = data.get("owner").ok_or("Missing owner field")?.clone();

        Ok(Room { id, title, owner })
    }
}