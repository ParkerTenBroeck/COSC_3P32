use std::time::SystemTime;

use crate::database::Db;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{post, routes};

use rocket_sync_db_pools::rusqlite;

use self::rusqlite::params;
use rocket::{delete, http::Status};

use super::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(super) struct SendMessage {
    message: String,
    reply: Option<i64>,
    attachment: Option<i64>,
    chat_id: i64,
}

#[post("/send_message", data = "<message>")]
async fn send_message(
    db: Db,
    user: users::UserId,
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
                    (sender_id, chat_id, message, attachment, posted, reply_to) 
                SELECT 
                    ?2, ?1, ?3, ?4, ?5, ?6
                WHERE 1=(
                    SELECT COUNT(*) FROM chat_members WHERE chat_id=?1 AND member_id=?2 
                    AND 1=(SELECT COUNT(*) FROM chats WHERE chat_id=?1 AND sending_privilage<=privilage) 
                ) AND IFNULL(?1=(
                    SELECT chat_id FROM messages WHERE reply_to=?6
                ), TRUE)
                RETURNING message_id",
                params![
                    message.chat_id,
                    user.0,
                    message.message,
                    message.attachment,
                    since_the_epoch as i64,
                    message.reply,
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
pub(super) struct MessageId {
    message_id: i64,
}

#[delete("/delete_message", data = "<message_id>")]
async fn delete_message(
    db: Db,
    user: users::UserId,
    message_id: Json<MessageId>,
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
    user: users::UserId,
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

#[post("/view_message", data = "<update>")]
async fn view_message(db: Db, _user: users::UserId, update: Json<MessageId>) -> Result<Status> {
    let updated = db
        .run(move |db| {
            db.execute(
                "
                UPDATE messages
                SET views=views+1
                WHERE message_id=?2
            ",
                params![update.message_id],
            )
        })
        .await?;
    match updated {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![send_message, delete_message, update_message, view_message]
}
