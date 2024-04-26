use crate::database::Db;
use rocket::serde::json::Json;
use rocket::{get, routes, Route};

use rocket_sync_db_pools::rusqlite;

use self::rusqlite::params;
use self::users::User;

use super::*;

#[get("/list_users")]
async fn list_users(db: Db) -> Result<Json<Vec<User>>> {
    let ids = db
        .run(|conn| {
            conn.prepare("SELECT * FROM users")?
                .query_map(params![], |row| {
                    Ok(User {
                        user_id: row.get(0)?,
                        phone_number: row.get(1)?,
                        name: row.get(2)?,
                        email: row.get(3)?,
                        location: row.get(4)?,
                        username: row.get(5)?,
                        password: row.get(6)?,
                        bio: row.get(7)?,
                        php_file_id: row.get(8)?,
                    })
                })?
                .collect::<Result<Vec<User>, _>>()
        })
        .await?;

    Ok(Json(ids))
}

pub fn routes() -> Vec<Route> {
    routes![list_users,]
}
