use crate::database::Db;
use rocket::response::{status::Created, Debug};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes};

use rocket_sync_db_pools::rusqlite;

use self::rusqlite::params;

type Result<T, E = Debug<rusqlite::Error>> = std::result::Result<T, E>;

pub mod user {

    #[derive(Debug, Clone, Copy)]
    pub(super) struct UserId(pub i64);

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for UserId {
        type Error = std::convert::Infallible;

        async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            request
                .cookies()
                .get_private("user_id")
                .and_then(|c| c.value().parse().ok())
                .map(|id| UserId(id))
                .or_forward(Status::Unauthorized)
        }
    }

    impl UserId {
        pub async fn load(self, conn: Db) -> Result<User, rusqlite::Error> {
            conn.run(move |db| {
                db.query_row(
                    "SELECT * FROM users WHERE user_id=?1",
                    params![self.0],
                    |row| {
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
                    },
                )
            })
            .await
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
            let user_id = match UserId::from_request(request).await {
                Outcome::Success(db) => db,
                Outcome::Forward(val) => return Outcome::Forward(val),
                _ => unreachable!(),
            };
            user_id.load(conn).await.or_forward(Status::Unauthorized)
        }
    }

    use rocket::{
        delete,
        http::{Cookie, CookieJar, Status},
        outcome::{IntoOutcome, Outcome},
        request::{self, FromRequest},
        Request, Route,
    };

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct CreateUser {
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
    ) -> Result<std::result::Result<Created<Json<i64>>, Status>> {
        let id = db
            .run(move |conn| {
                conn.query_row(
                    "
                INSERT INTO users 
                    (phone_number, name, email, location, username, password, availability) 
                VALUES 
                    (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                RETURNING user_id",
                    params![
                        create_user.phone_number,
                        create_user.name,
                        create_user.email,
                        create_user.location,
                        create_user.username,
                        create_user.password,
                        2
                    ],
                    |r| r.get::<_, i64>(0),
                )
            })
            .await;

        match id {
            Ok(id) => Ok(Ok(Created::new("/").body(id.into()))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Err(Status::Conflict)),
            Err(err) => Err(err.into()),
        }
    }

    fn deserialize_optional_field<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
    where
        D: rocket::serde::Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Deserialize::deserialize(de).map(Some)
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    struct UpdateUser {
        #[serde(skip_serializing_if = "Option::is_none")]
        phone_number: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        location: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bio: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        availability: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(deserialize_with = "deserialize_optional_field")]
        pfp_file_id: Option<Option<i64>>,
    }

    #[post("/update_user", data = "<updates>")]
    async fn update_user(db: Db, user: UserId, updates: Json<UpdateUser>) -> Result<Status> {
        todo!();

        let affected = db
            .run(move |db| {
                Result::<_, rusqlite::Error>::Ok(db.execute(
                    "
                UPDATE users
                SET
                WHERE user_id=?1
            ",
                    params![user.0],
                )?)
            })
            .await?;

        match affected {
            1 => Ok(Status::Accepted),
            _ => Ok(Status::NotAcceptable),
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

        match id {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AuthResult::Failed.into()),
            Ok(id) => {
                jar.add_private(Cookie::build(("user_id", format!("{id}"))));
                Ok(AuthResult::Authorized.into())
            }
            Err(err) => Err(err.into()),
        }
    }

    #[post("/logout")]
    async fn logout(jar: &CookieJar<'_>) {
        jar.remove_private("user_id")
    }

    #[get("/who_am_i")]
    async fn who_am_i(user: Option<User>) -> Result<Json<Option<User>>> {
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
    async fn delete_account(db: Db, jar: &CookieJar<'_>, user: UserId) -> Result<Status> {
        let affected = db
            .run(move |conn| conn.execute("DELETE FROM users WHERE user_id=?1", params![user.0]))
            .await?;

        jar.remove_private("user_id");

        match affected {
            0 => Ok(Status::InternalServerError),
            1 => Ok(Status::Accepted),
            _ => Ok(Status::BadRequest),
        }
    }

    pub fn routes() -> Vec<Route> {
        routes![
            new_user,
            list_users,
            login,
            who_am_i,
            logout,
            delete_account,
            update_user
        ]
    }
}

mod chats {
    use rocket::{delete, http::Status};

    use super::*;

    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    struct CreateDM {
        other: i64,
    }

    #[post("/create_dm", data = "<dm>")]
    async fn create_dm(db: Db, user: user::UserId, dm: Json<CreateDM>) -> Result<Json<i64>> {
        let val = db.run(move |db|{
            let tran = db.transaction()?;
            
            let chat_id = tran.query_row("
            INSERT INTO chats
                (primary_owner, secondary_owner, sending_privilage, track_views, max_members)
            SELECT
                ?1, ?2, 0, FALSE, 2
            WHERE NOT EXISTS 
                (SELECT 1 FROM chats WHERE (primary_owner=?1 AND secondary_owner=?2) OR (primary_owner=?2 AND secondary_owner=?1))
            RETURNING chat_id", params![user.0, dm.other], |row| Ok(row.get(0)?))?;
            
            tran.execute("
            INSERT INTO chat_members
                (chat_id, member_id, privilage)
            VALUES
                (?1, ?2, 255)", params![chat_id, user.0])?;
            
            tran.execute("
            INSERT INTO chat_members
                (chat_id, member_id, privilage)
            VALUES
                (?1, ?2, 255)", params![chat_id, dm.other])?;

            tran.commit()?;

            Result::<i64, rusqlite::Error>::Ok(chat_id)
        }).await?;
        Ok(val.into())
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    struct CreateGroup {
        name: Option<String>,
    }

    #[post("/create_group", data = "<group>")]
    async fn create_group(
        db: Db,
        user: user::UserId,
        group: Json<CreateGroup>,
    ) -> Result<Json<i64>> {
        let val = db
            .run(move |db| {
                let tran = db.transaction()?;

                let chat_id = tran.query_row(
                    "
            INSERT INTO chats
                (primary_owner, sending_privilage, track_views, max_members, chat_name)
            SELECT
                ?1, 0, FALSE, 2000, ?2
            WHERE 100>(
                SELECT COUNT(*) FROM 
                    (SELECT chat_id FROM chats WHERE secondary_owner IS NULL) t1
                LEFT JOIN
                    chat_members
                ON (t1.chat_id=chat_members.chat_id)
                WHERE member_id=?1
            )
            RETURNING chat_id",
                    params![user.0, group.name],
                    |row| Ok(row.get(0)?),
                )?;

                tran.execute(
                    "
            INSERT INTO chat_members
                (chat_id, member_id, privilage)
            VALUES
                (?1, ?2, 255)",
                    params![chat_id, user.0],
                )?;

                tran.commit()?;

                Result::<i64, rusqlite::Error>::Ok(chat_id)
            })
            .await?;
        Ok(val.into())
    }

    #[post("/create_channel", data = "<group>")]
    async fn create_channel(
        db: Db,
        user: user::UserId,
        group: Json<CreateGroup>,
    ) -> Result<Json<i64>> {
        let val = db
            .run(move |db| {
                let tran = db.transaction()?;

                let chat_id = tran.query_row(
                    "
            INSERT INTO chats
                (primary_owner, secondary_owner, sending_privilage, track_views, max_members, chat_name)
            SELECT
                ?1, ?2, 128, TRUE, 18446744073709551615, ?2
            WHERE 100>(
                SELECT COUNT(*) FROM 
                    (SELECT chat_id FROM chats WHERE secondary_owner IS NULL) t1
                LEFT JOIN
                    chat_members
                ON (t1.chat_id=chat_members.chat_id)
                WHERE member_id=?1
            )
            RETURNING chat_id",
                    params![user.0, group.name],
                    |row| Ok(row.get(0)?),
                )?;

                tran.execute(
                    "
            INSERT INTO chat_members
                (chat_id, member_id, privilage)
            VALUES
                (?1, ?2, 255)",
                    params![chat_id, user.0],
                )?;

                tran.commit()?;

                Result::<i64, rusqlite::Error>::Ok(chat_id)
            })
            .await?;
        Ok(val.into())
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    struct ChatId {
        chat_id: i64,
    }

    #[post("/join_chat", data = "<chat>")]
    async fn join_chat(db: Db, user: user::UserId, chat: Json<ChatId>) -> Result<Status> {
        let affected =
            db.0.run(move |db| {
                Result::<_, rusqlite::Error>::Ok(db.execute(
                    "
            INSERT INTO chat_members
                (chat_id, member_id, privilage)
            SELECT
                ?1, ?2, 0
            WHERE 
            100>(
                SELECT COUNT(*) FROM 
                    (SELECT chat_id FROM chats WHERE secondary_owner IS NULL) t1
                LEFT JOIN
                    chat_members
                ON (t1.chat_id=chat_members.chat_id)
                WHERE member_id=?1
            ) AND
            (
                (SELECT SUM(max_members) FROM chats WHERE chat_id=?1)
                > 
                (SELECT COUNT(*) FROM chat_members WHERE chat_id=?1)
            )
            ",
                    params![chat.chat_id, user.0],
                )?)
            })
            .await?;

        match affected {
            1 => Ok(Status::Accepted),
            _ => Ok(Status::NotAcceptable),
        }
    }

    #[post("/leave_chat", data = "<chat>")]
    async fn leave_chat(db: Db, user: user::UserId, chat: Json<ChatId>) -> Result<Status> {
        let num = db
            .run(move |db| {
                db.execute(
                    "
            DELETE FROM chat_members
                WHERE 
            chat_id=?1 AND member_id=?2 
            AND IFNULL(?2 NOT IN(
                SELECT 
                    primary_owner 
                FROM 
                    chats 
                WHERE 
                    chats.chat_id=?1
            ), TRUE)
            AND IFNULL(?2 NOT IN(
                SELECT 
                    secondary_owner 
                FROM 
                    chats 
                WHERE 
                    chats.chat_id=?1
            ), TRUE)",
                    params![chat.chat_id, user.0],
                )
            })
            .await?;

        match num {
            1 => Ok(Status::Ok),
            _ => Ok(Status::NotAcceptable),
        }
    }

    #[delete("/delete_chat", data = "<chat>")]
    async fn delete_chat(db: Db, user: user::UserId, chat: Json<ChatId>) -> Result<Status> {
        let affected =
            db.0.run(move |db| {
                Result::<_, rusqlite::Error>::Ok(db.execute(
                    "
            DELETE FROM chats
                WHERE
                chat_id = ?2 AND (primary_owner=?1 OR IFNULL(secondary_owner=?1, FALSE))
            ",
                    params![user.0, chat.chat_id],
                )?)
            })
            .await?;

        match affected {
            1 => Ok(Status::Accepted),
            _ => Ok(Status::Unauthorized),
        }
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    struct UpdatePerm {
        chat_id: i64,
        user_id: i64,
        new_perm: u8,
    }

    #[delete("/update_chat_member_perm", data = "<updated>")]
    async fn update_chat_member_perm(
        db: Db,
        user: user::UserId,
        updated: Json<UpdatePerm>,
    ) -> Result<Status> {
        let affected = db.0.run(move |db|{
            Result::<_, rusqlite::Error>::Ok(db.execute("
            UPDATE chat_members
                SET privilage=?3
            WHERE
                chat_id=?1 AND member_id=?2 
                AND ?3<(SELECT SUM(chat_members) FROM chat_members WHERE chat_id=?1 AND member_id=?4)
                AND privilage<(SELECT SUM(chat_members) FROM chat_members WHERE chat_id=?1 AND member_id=?4)
            ", params![updated.chat_id, updated.user_id, updated.new_perm, user.0])?)
        }).await?;

        match affected {
            1 => Ok(Status::Accepted),
            _ => Ok(Status::Unauthorized),
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct Chat {
        chat_id: i64,
        owner: i64,
        seconary: Option<i64>,
        send_priv: u8,
        tracks_views: bool,
        max_members: u64,
        name: Option<String>,
    }

    #[get("/list_chats")]
    async fn list_chats(db: Db, user: user::UserId) -> Result<Json<Vec<Chat>>> {
        Ok(db
            .run(move |db| {
                db.prepare(
                    "
            SELECT chats.*
            FROM chats
            INNER JOIN chat_members ON chats.chat_id=chat_members.chat_id
            AND chat_members.member_id=?1
            ",
                )?
                .query_map(params![user.0], |row| {
                    Ok(Chat {
                        chat_id: row.get(0)?,
                        owner: row.get(1)?,
                        seconary: row.get(2)?,
                        send_priv: row.get(3)?,
                        tracks_views: row.get(4)?,
                        max_members: row.get(5)?,
                        name: row.get(6)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .await?
            .into())
    }

    #[post("/list_chat_members", data = "<chat>")]
    async fn list_chat_members(db: Db, user: user::UserId, chat: Json<ChatId>) -> Result<Json<Vec<i64>>> {
        Ok(db
            .run(move |db| {
                db.prepare(
                    "
            SELECT member_id
            FROM chat_members
            WHERE ?1=(SELECT member_id FROM chat_members WHERE chat_id=?2 AND member_id=?1)
                    AND ?1!=member_id AND chat_id=?2
            ",
                )?
                .query_map(params![user.0, chat.chat_id], |row| {
                    Ok(row.get(0)?)
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .await?
            .into())
    }

    pub fn routes() -> Vec<rocket::Route> {
        routes![
            create_dm,
            create_channel,
            create_group,
            join_chat,
            delete_chat,
            leave_chat,
            update_chat_member_perm,
            list_chats,
            list_chat_members
        ]
    }
}

mod messages {
    use std::time::SystemTime;

    use rocket::{delete, http::Status};

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct SendMessage {
        message: String,
        attachment: Option<i64>,
        chat_id: i64,
    }

    #[post("/send_message", data = "<message>")]
    async fn send_message(
        db: Db,
        user: user::UserId,
        message: Json<SendMessage>,
    ) -> Result<Json<i64>> {
        let since_the_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mid = db
            .run(move |db| {
                let tran = db.transaction()?;
                
                let mid: i64 = tran.query_row(
                    "
                    INSERT INTO messages 
                        (sender_id, chat_id, message, attachment, posted) 
                    SELECT 
                        ?2, ?1, ?3, ?4, ?5
                    WHERE 1=(
                        SELECT COUNT(*) FROM chat_members WHERE chat_id=?1 AND member_id=?2 
                        AND 1=(SELECT COUNT(*) FROM chats WHERE chat_id=?1 AND sending_privilage<=privilage) 
                    ) RETURNING message_id",
                    params![
                        message.chat_id,
                        user.0,
                        message.message,
                        message.attachment,
                        since_the_epoch as i64
                    ],
                    |row| row.get(0),
                )?;
                tran.commit()?;
                Result::<_, rusqlite::Error>::Ok(mid)
            })
            .await?;

        Ok(mid.into())
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct DeleteMessage {
        message_id: i64,
    }

    #[delete("/delete_message", data = "<message_id>")]
    async fn delete_message(
        db: Db,
        user: user::UserId,
        message_id: Json<DeleteMessage>,
    ) -> Result<Status> {
        let affected = db
            .run(move |conn| {
                conn.execute(
                    "
            DELETE FROM messages 
            WHERE message_id=?1 AND owner_id=?2",
                    params![message_id.message_id, user.0],
                )
            })
            .await?;

        match affected {
            0 => Ok(Status::InternalServerError),
            1 => Ok(Status::Accepted),
            _ => Ok(Status::BadRequest),
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(crate = "rocket::serde")]
    pub(super) struct UpdateMessage {
        message_id: i64,
        message: String,
    }

    #[post("/update_message", data = "<message>")]
    async fn update_message(
        db: Db,
        user: user::UserId,
        message: Json<UpdateMessage>,
    ) -> Result<Status> {
        let affected = db
            .run(move |conn| {
                let since_the_epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                let tran = conn.transaction()?;
                let updated = tran.execute(
                    "
                UPDATE messages
                SET message = ?3, last_edited=?4
                WHERE message_id=?1 AND sender_id=?2",
                    params![
                        message.message_id,
                        user.0,
                        message.message,
                        since_the_epoch as i64
                    ],
                )?;
                if updated != 1 {
                    tran.rollback()?;
                } else {
                    tran.commit()?;
                }
                Result::<_, rusqlite::Error>::Ok(updated)
            })
            .await?;

        match affected {
            0 => Ok(Status::InternalServerError),
            1 => Ok(Status::Accepted),
            _ => Ok(Status::BadRequest),
        }
    }

    pub fn routes() -> Vec<rocket::Route> {
        routes![send_message, delete_message, update_message]
    }
}

pub fn api_routes() -> Vec<rocket::Route> {
    let mut items = Vec::new();
    items.append(&mut user::routes());
    items.append(&mut messages::routes());
    items.append(&mut chats::routes());
    items
}
