pub mod admin;
pub mod chats;
pub mod messages;
pub mod users;

use rocket::response::Debug;
use rocket::{fairing::AdHoc, Build, Rocket};
use rocket_sync_db_pools::database;

type Result<T, E = Debug<rusqlite::Error>> = std::result::Result<T, E>;

#[database("rusqlite")]
pub(in crate::database) struct Db(rusqlite::Connection);

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    Db::get_one(&rocket)
        .await
        .expect("database mounted")
        .run(move |conn| conn.execute_batch(include_str!("../../db/rusqlite/migrations/up.sql")))
        .await
        .expect("can init rusqlite DB");

    rocket
}

pub fn stage_database() -> AdHoc {
    AdHoc::on_ignite("Rusqlite Stage", |rocket| async {
        let exists = std::path::Path::new("rusqlite.db").exists();

        let rocket = rocket.attach(Db::fairing());

        if !exists {
            rocket.attach(AdHoc::on_ignite("Rusqlite Init", init_db))
        } else {
            rocket
        }
        .mount("/database", users::routes())
        .mount("/database", messages::routes())
        .mount("/database", chats::routes())
        .mount("/database", admin::routes())
    })
}
