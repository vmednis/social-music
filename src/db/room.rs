use crate::db;
use redis::{AsyncCommands, Commands};
use regex::Regex;
use std::collections::HashSet;

impl db::DbInternal {
    fn key_room(room_id: String) -> String {
        format!("room:{}", room_id)
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
}

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

        args
    }
}
