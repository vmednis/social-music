mod cookie;
mod db;
mod endpoint;
mod room;
mod routes;
mod socket;
mod spotify;

#[tokio::main]
async fn main() {
    env_logger::init();

    let db = db::connect_db();
    let spotify = spotify::init(db.clone());

    room::start_listener(db.clone(), spotify.clone()).await;
    warp::serve(routes::routes(db, spotify))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
