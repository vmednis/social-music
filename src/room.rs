use crate::db;
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
                let time = play_next_song(db.clone(), spotify.clone(), room_id.clone()).await.unwrap_or(2000);
                play_song.as_mut().reset(tokio::time::Instant::now() + tokio::time::Duration::from_millis(time));
            }
        }
    }
}

async fn play_next_song(db: db::Db, spotify: spotify::Spotify, room_id: String) -> Option<u64> {
    let mut db = db.lock().await;
    if let Some(next_user_id) = db.pop_queue(room_id.clone()).await {
        if let Some(uri) = db.pop_user_queue(room_id.clone(), next_user_id.clone()).await {
            let spotify = spotify.lock().await;
            let token = db.get_auth(next_user_id.clone()).await.unwrap();
            let device_id = db.get_device(next_user_id.clone()).await.unwrap();
            let track = spotify.request_track(token.clone(), uri.clone()).await;
            spotify.request_play(token, device_id, uri, 0).await;

            db.push_queue(room_id, next_user_id).await;

            Some(track.duration_ms)
        } else {
            None
        }
    } else {
        None
    }
}