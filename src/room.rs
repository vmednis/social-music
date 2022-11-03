use crate::db;
use crate::spotify;
use crate::db::presence::PresenceEventActivty;

pub async fn start_listener(db: db::Db, spotify: spotify::Spotify) {
    tokio::task::spawn(async move {
        loop {
            let mut inner_db = db.lock().await;
            let rx = inner_db.claim_room().await;
            std::mem::drop(inner_db);

            if let Some(room_id) = rx.await.unwrap() {
                tokio::task::spawn(serve_room(db.clone(), spotify.clone(), room_id));
            }
        }
    });
}

async fn serve_room(db: db::Db, spotify: spotify::Spotify, room_id: String) {
    log::info!("Claimed room {}", room_id.clone());

    let (_kill_presence_tx, mut kill_presence_rx) = tokio::sync::mpsc::channel::<()>(1);
    let inner_db = db.clone();
    let inner_room_id = room_id.clone();
    tokio::task::spawn(async move {
        let mut db = inner_db.lock().await;
        let mut presence_rx = db.subscribe_presence(inner_room_id.clone()).await;
        let users = db.scan_presence(inner_room_id.clone()).await;
        db.del_presences(inner_room_id.clone()).await;
        for user_id in users {
            db.add_presences(inner_room_id.clone(), user_id).await;
        }
        std::mem::drop(db);

        loop {
            tokio::select! {
                presence_event = presence_rx.recv() => {
                    match presence_event {
                        Some(event) => {
                            let mut db = inner_db.lock().await;
                            match event.activity {
                                PresenceEventActivty::Join => {
                                    db.add_presences(inner_room_id.clone(), event.user_id).await;
                                },
                                PresenceEventActivty::Leave => {
                                    db.rem_presences(inner_room_id.clone(), event.user_id).await;
                                }
                            }
                        },
                        None => {
                            break;
                        }
                    }
                },
                _ = kill_presence_rx.recv() => {
                    break;
                }
            }
        }
    });

    let refresh = tokio::time::sleep(tokio::time::Duration::from_secs(3));
    let play_song = tokio::time::sleep(tokio::time::Duration::from_secs(15));

    tokio::pin!(refresh);
    tokio::pin!(play_song);

    loop {
        tokio::select! {
            _ = &mut refresh => {
                let mut db = db.lock().await;
                db.keep_alive_room_claim(room_id.clone()).await;
                refresh.as_mut().reset(tokio::time::Instant::now() + tokio::time::Duration::from_secs(3));
            }
            _ = &mut play_song => {
                let time = play_next_song(db.clone(), spotify.clone(), room_id.clone()).await.unwrap_or(15000);
                play_song.as_mut().reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(time));
            }
        }
    }

    //kill_presence_tx.send(()).await.unwrap();
}

async fn play_next_song(db: db::Db, spotify: spotify::Spotify, room_id: String) -> Option<u64> {
    let mut inner_db = db.lock().await;
    if let Some(next_user_id) = inner_db.pop_queue(room_id.clone()).await {
        if let Some(uri) = inner_db.pop_user_queue(room_id.clone(), next_user_id.clone()).await {
            let token = inner_db.get_auth(next_user_id.clone()).await.unwrap();
            let users = inner_db.list_presences(room_id.clone()).await;
            inner_db.push_queue(room_id, next_user_id).await;
            std::mem::drop(inner_db);

            for user_id in users {
                play_song(db.clone(), spotify.clone(), user_id, uri.clone(), 0).await;
            }

            let spotify = spotify.lock().await;
            let track = spotify.request_track(token.clone(), uri.clone()).await;

            Some(track.duration_ms)
        } else {
            None
        }
    } else {
        None
    }
}

async fn play_song(db: db::Db, spotify: spotify::Spotify, user_id: String, uri: String, position: u32) {
    let mut db = db.lock().await;
    let token = db.get_auth(user_id.clone()).await.unwrap();
    let device_id = db.get_device(user_id.clone()).await.unwrap();
    std::mem::drop(db);

    let spotify = spotify.lock().await;
    spotify.request_play(token, device_id, uri, position).await;
}