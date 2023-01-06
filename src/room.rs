use crate::db;
use crate::db::message::Message;
use crate::db::presence::PresenceEventActivty;
use crate::spotify;

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
            db.add_presences(inner_room_id.clone(), user_id.clone())
                .await;
            db.add_message(inner_room_id.clone(), Message::presence_changed())
                .await;
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
                                    db.add_presences(inner_room_id.clone(), event.user_id.clone()).await;
                                    db.add_message(inner_room_id.clone(), Message::presence_changed()).await;
                                },
                                PresenceEventActivty::Leave => {
                                    db.rem_presences(inner_room_id.clone(), event.user_id.clone()).await;
                                    db.rem_queue(inner_room_id.clone(), event.user_id).await;
                                    db.add_message(inner_room_id.clone(), Message::presence_changed()).await;
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

    let (_kill_db_tx, mut kill_db_rx) = tokio::sync::mpsc::channel::<()>(1);
    let inner_db = db.clone();
    let inner_room_id = room_id.clone();
    let inner_spotify = spotify.clone();
    tokio::task::spawn(async move {
        let mut db = inner_db.lock().await;
        let mut db_rx = db.subscribe_messages(inner_room_id.clone()).await;
        std::mem::drop(db);

        loop {
            tokio::select! {
                message = db_rx.recv() => {
                    if let Some(message) = message {
                        match message.data {
                            db::message::MessageType::MessageDeviceChange(device_change) => {
                                play_song_on_join(inner_db.clone(), inner_spotify.clone(), inner_room_id.clone(), device_change.user_id).await;
                            },
                            _ => ()
                        }
                    }
                },
                _ = kill_db_rx.recv() => {
                    break;
                }
            }
        }
    });

    let refresh = tokio::time::sleep(tokio::time::Duration::from_secs(3));
    let play_song = tokio::time::sleep(tokio::time::Duration::from_secs(1));

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
                let time = play_next_song(db.clone(), spotify.clone(), room_id.clone()).await.unwrap_or(1000);
                play_song.as_mut().reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(time));
            }
        }
    }
}

async fn play_next_song(db: db::Db, spotify: spotify::Spotify, room_id: String) -> Option<u64> {
    let mut inner_db = db.lock().await;
    if let Some(next_user_id) = inner_db.pop_queue(room_id.clone()).await {
        if let Some(uri) = inner_db
            .pop_user_queue(room_id.clone(), next_user_id.clone())
            .await
        {
            //Notify next_user_id on their queue change
            inner_db.add_message(room_id.clone(), Message::user_queue_changed(next_user_id.clone())).await;

            //Gather all we need to play the next song
            let token = inner_db.get_auth(next_user_id.clone()).await.unwrap();
            let users = inner_db.list_presences(room_id.clone()).await;
            inner_db.push_queue(room_id.clone(), next_user_id).await;
            std::mem::drop(inner_db);

            let inner_spotify = spotify.lock().await;
            let track = inner_spotify
                .request_track(token.clone(), uri.clone())
                .await;
            std::mem::drop(inner_spotify);

            let mut inner_db = db.lock().await;
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            inner_db
                .set_playing(
                    room_id.clone(),
                    uri.clone(),
                    time,
                    track.duration_ms.clone(),
                )
                .await;
            inner_db
                .add_message(room_id.clone(), Message::queue_changed())
                .await;
            std::mem::drop(inner_db);

            for user_id in users {
                play_song(db.clone(), spotify.clone(), user_id, uri.clone(), 0).await;
            }

            Some(track.duration_ms)
        } else {
            inner_db
                .add_message(room_id.clone(), Message::queue_changed())
                .await;
            None
        }
    } else {
        inner_db
            .add_message(room_id.clone(), Message::queue_changed())
            .await;
        None
    }
}

async fn play_song(
    db: db::Db,
    spotify: spotify::Spotify,
    user_id: String,
    uri: String,
    position: u32,
) {
    let mut db = db.lock().await;
    let token = db.get_auth(user_id.clone()).await.unwrap();
    let device_id = db.get_device(user_id.clone()).await.unwrap();
    std::mem::drop(db);

    let spotify = spotify.lock().await;
    spotify.request_play(token, device_id, uri, position).await;
}

async fn play_song_on_join(
    db: db::Db,
    spotify: spotify::Spotify,
    room_id: String,
    user_id: String,
) {
    //This function is called when the device id changes (i.e. user rejoins the room in the middle of song playback)
    //since spotify systems are a bit weird there seems to be a race condition where we have a device id, but it hasn't
    //registered with the rest of the spotify systems. We have to wait a bit. Sleeping for 1 second seemed to be 50/50
    //chance of it going through, so chose 3 seconds to be safe.
    //TODO: Replace with a retry loop
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let mut inner_db = db.lock().await;
    let playing = inner_db.get_playing(room_id).await;
    std::mem::drop(inner_db);

    if let Some(playing) = playing {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        if time < playing.start_time + u128::from(playing.length) {
            let offset = u32::try_from(time - playing.start_time).unwrap();
            play_song(db.clone(), spotify, user_id, playing.track_id, offset).await;
        }
    }
}
