use crate::database::Db;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes};

use rocket_sync_db_pools::rusqlite;

use self::rusqlite::params;

type Result<T, E = Debug<rusqlite::Error>> = std::result::Result<T, E>;

pub mod user {

    #[derive(Clone, Copy)]
    pub struct UserId(pub i64);

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for UserId {
        type Error = std::convert::Infallible;

        async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            request.cookies()
                .get_private("user_id")
                .and_then(|c| c.value().parse().ok())
                .map(|id| UserId(id))
                .or_forward(Status::Unauthorized)
        }
    }

    impl UserId{
        pub async fn load(self, conn: Db) -> Result<User, rusqlite::Error>{
            conn.run(move |db|{
                db.query_row("SELECT * FROM users WHERE user_id=?1", params![self.0], |row| Ok(
                    User{
                        user_id: row.get(0)?,
                        phone_number: row.get(1)?,
                        name: row.get(2)?,
                        email: row.get(3)?,
                        location: row.get(4)?,
                        username: row.get(5)?,
                        password: row.get(6)?,
                        bio: row.get(7)?,
                        php_file_id: row.get(8)?, 
                    }
                ))
            }).await
        }
    }

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for User {
        type Error = rusqlite::Error;

        async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            let conn = match Db::get_one(request.rocket())
                .await
                .or_forward(Status::Unauthorized)
            {
                Outcome::Success(db) => db,
                Outcome::Error(val) => return Outcome::Error(val),
                Outcome::Forward(val) => return Outcome::Forward(val),
            };
            let user_id = match UserId::from_request(request).await
            {
                Outcome::Success(db) => db,
                Outcome::Forward(val) => return Outcome::Forward(val),
                _ => unreachable!()
            };
            user_id.load(conn).await.or_forward(Status::Unauthorized)
        }
    }

    use rocket::{
        delete, http::{Cookie, CookieJar, Status}, outcome::{IntoOutcome, Outcome}, request::{self, FromRequest}, Request, Route
    };

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct CreateUser {
        #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
        user_id: Option<i64>,
        phone_number: String,
        name: String,
        email: String,
        location: String,
        username: String,
        password: String,
    }

    #[post("/create_user", data = "<create_user>")]
    pub(super) async fn new_user(
        db: Db,
        create_user: Json<CreateUser>,
    ) -> Result<std::result::Result<Created<Json<CreateUser>>, Status>> {
        if 
            create_user.email.is_empty() 
            | create_user.name.is_empty()
            | create_user.password.is_empty()
            | create_user.phone_number.is_empty()
            | create_user.username.is_empty()
            | create_user.location.is_empty(){
            return Ok(Err(Status::BadRequest))
        }
        let mut created_user = create_user.clone();
        let id = db.run(move |conn| {
            conn.query_row("
                INSERT INTO users 
                    (phone_number, name, email, location, username, password, availability) 
                        SELECT 
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7 
                WHERE NOT EXISTS (
                    SELECT 1 FROM users WHERE email =?3 OR phone_number =?1
                ) RETURNING user_id",
                params![create_user.phone_number, create_user.name, create_user.email, create_user.location, create_user.username, create_user.password, 2],
                |r| r.get(0))
        }).await;

        match id{
            Ok(id) => {
                created_user.user_id = Some(id);
                Ok(Ok(Created::new("/").body(created_user)))
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Err(Status::Conflict)),
            Err(err) => Err(err.into())
        }

    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct LoginRequest {
        email: String,
        password: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    enum AuthResult {
        Authorized,
        Failed,
    }

    #[post("/login", data = "<login>")]
    async fn login(
        db: Db,
        jar: &CookieJar<'_>,
        login: Json<LoginRequest>,
    ) -> Result<Json<AuthResult>> {

        jar.remove_private("user_id");
        
        let id: Result<i64, _> = db
            .run(move |conn| {
                conn.query_row(
                    "SELECT user_id FROM users WHERE email=?1 AND password=?2",
                    params![login.email, login.password],
                    |r| r.get(0),
                )
            })
            .await;

        match id{
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AuthResult::Failed.into()),
            Ok(id) => {
                jar.add_private(Cookie::build(("user_id", format!("{id}"))));
                Ok(AuthResult::Authorized.into())
            }
            Err(err) => Err(err.into())
        }
    }

    #[post("/logout")]
    async fn logout(jar: &CookieJar<'_>) {
        jar.remove_private("user_id")
    }

    #[get("/who_am_i")]
    async fn who_am_i(
        user: Option<User>,
    ) -> Result<Json<Option<User>>> {
        Ok(user.into())
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub struct User {
        #[serde(skip_deserializing)]
        user_id: i64,
        phone_number: String,
        name: String,
        email: String,
        location: String,
        username: String,
        password: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        bio: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        php_file_id: Option<i64>,
    }

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

    #[delete("/delete_account")]
    async fn delete_account(db: Db, jar: &CookieJar<'_>, user: UserId) -> Result<Status>{
        let affected = db
            .run(move |conn| {
                conn.execute("DELETE FROM users WHERE user_id=?1", params![user.0])
            })
            .await?;

        jar.remove_private("user_id");

        match affected{
            0 => Ok(Status::InternalServerError),
            1 => Ok(Status::Accepted),
            _ => Ok(Status:: BadRequest),
        }
    }

    pub fn routes() -> Vec<Route> {
        routes![new_user, list_users, login, who_am_i, logout, delete_account]
    }
}

mod messages{
    use std::time::SystemTime;

    use rocket::{delete, http::Status};

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct CreateMessage {
        message: String,
        attachment: Option<i64>,
        to: i64,
    }

    #[post("/private_message", data = "<message>")]
    async fn private_message(db: Db, user: user::UserId, message: Json<CreateMessage>) -> Result<Json<i64>>{
        
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        let mid = db.run(move |db|{
            let tran = db.transaction()?;
            // tran.query_row(sql, params, f)
            let mid: i64 = tran.query_row("
                INSERT INTO messages 
                    (message, attachment, posted) 
                VALUES 
                    (?1, ?2, ?3) 
                RETURNING message_id", 
                params![message.message, message.attachment, since_the_epoch as i64], |row|{
                    row.get(0)
            })?;

            tran.execute("
                INSERT INTO private_messages (from_id, to_id, message_id) VALUES (?1, ?2, ?3)", params![user.0, message.to, mid])?;
            
            tran.execute("UPDATE messages SET private_message_id=?1 WHERE message_id=?1", params![mid])?;
            
            tran.commit()?;
            Result::<_, rusqlite::Error>::Ok(mid)
        }).await?;

        Ok(mid.into())
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct DeleteMessage {
        message_id: i64
    }


    #[delete("/delete_message", data = "<message_id>")]
    async fn delete_message(db: Db, user: user::UserId, message_id: Json<DeleteMessage>) -> Result<Status>{
        let affected = db
        .run(move |conn| {
            conn.execute("
            DELETE FROM messages 
            WHERE message_id=?1 AND owner_id=?2", params![message_id.message_id, user.0])
        })
        .await?;


        match affected{
            0 => Ok(Status::InternalServerError),
            1 => Ok(Status::Accepted),
            _ => Ok(Status:: BadRequest),
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct UpdateMessage {
        message_id: i64,
        message: String,
    }

    #[post("/update_message", data = "<message>")]
    async fn update_message(db: Db, user: user::UserId, message: Json<UpdateMessage>) -> Result<Status>{
        let affected = db
        .run(move |conn| {
            let tran = conn.transaction()?;
            let updated = tran.execute("
                UPDATE messages
                SET message = ?3 
                WHERE message_id=?1 owner_id=?2", 
                params![message.message_id, user.0, message.message]
            )?;
            if updated != 1{
                tran.rollback()?;
            }else{
                tran.commit()?;
            }
            Result::<_, rusqlite::Error>::Ok(updated)
        })
        .await?;


        match affected{
            0 => Ok(Status::InternalServerError),
            1 => Ok(Status::Accepted),
            _ => Ok(Status:: BadRequest),
        }
    }

    pub fn routes() -> Vec<rocket::Route>{
        routes![private_message, delete_message, update_message]
    }
}

pub fn api_routes() -> Vec<rocket::Route> {
    let mut items = Vec::new();
    items.append(&mut user::routes());
    items.append(&mut messages::routes());
    items
}
