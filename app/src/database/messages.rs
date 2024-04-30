use std::time::SystemTime;

use crate::database::Db;
use crate::make_id;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes, State};

use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use self::chats::ChatId;
use self::rusqlite::params;
use self::sessions::{SessionEvent, SessionManager};
use self::users::*;
use rocket::{delete, http::Status};

use super::*;


make_id!(MessageId);



#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(super) struct SendMessage {
    message: String,
    reply: Option<MessageId>,
    attachment: Option<i64>,
    chat_id: ChatId,
}

#[post("/send_message", data = "<message>")]
async fn send_message(
    db: Db,
    user: UserLoggedIn,
    message: Json<SendMessage>,
    sessions: &State<SessionManager>
) -> Result<Json<MessageId>> {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let chat = message.chat_id;

    let mid = db
        .run(move |db| {
            let tran = db.transaction()?;

            let mid = tran.query_row(
                "
                INSERT INTO messages 
                    (sender_id, chat_id, message, attachment, posted, reply_to) 
                SELECT 
                    :user_id, :chat_id, :message, :attachment, :posted, :reply_to
                WHERE 1=(
                    SELECT COUNT(*) FROM chat_members WHERE chat_id=:chat_id AND member_id=:user_id 
                    AND 1=(SELECT COUNT(*) FROM chats WHERE chat_id=:chat_id AND sending_privilage<=privilage) 
                ) AND IFNULL(:chat_id=(
                    SELECT chat_id FROM messages WHERE reply_to=:reply_to
                ), TRUE)
                RETURNING message_id",
                named_params![
                    ":chat_id": message.chat_id,
                    ":user_id": user,
                    ":message": message.message,
                    ":attachment": message.attachment,
                    ":posted": since_the_epoch as i64,
                    ":reply_to": message.reply,
                ],
                |row| row.get(0),
            )?;
            tran.commit()?;
            Result::<_, rusqlite::Error>::Ok(mid)
        })
        .await?;

        sessions.event(SessionEvent::NewMessage { chat, message: mid }).await;

    Ok(mid.into())
}

#[delete("/delete_message/<message>")]
async fn delete_message(
    db: Db,
    user: UserLoggedIn,
    message: MessageId,
    sessions: &State<SessionManager>
) -> Result<Status> {
    let chat: ChatId = db
        .run(move |conn| {
            conn.query_row(
                "
        DELETE FROM messages 
        WHERE message_id=:message_id AND 
        (
            sender_id=:user_id
            OR 
            (SELECT privilage FROM chat_members WHERE member_id=:user_id)
            >
            (SELECT privilage FROM chat_members WHERE member_id=message_id)
        )
        RETURNING chat_id",
                named_params![
                    ":message_id": message, 
                    ":user_id": user],
            |row| row.get(0))
        })
        .await?;

    sessions.event(SessionEvent::MessageDeleted { chat, message }).await;
    Ok(Status::Accepted)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(super) struct UpdateMessage {
    message: String,
}

#[post("/update_message/<message>", data = "<content>")]
async fn update_message(
    db: Db,
    user: UserLoggedIn,
    message: MessageId,
    content: Json<UpdateMessage>,
    sessions: &State<SessionManager>
) -> Result<Status> {
    let chat = db
        .run(move |conn| {
            let since_the_epoch = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            let tran = conn.transaction()?;
            let updated = tran.query_row(
                "
            UPDATE messages
            SET message = ?3, last_edited=?4
            WHERE message_id=?1 AND sender_id=?2
            RETURNING chat_id",
                params![
                    message,
                    user,
                    content.message,
                    since_the_epoch as i64
                ], |row| row.get(0)
            )?;
            
            tran.commit()?;
            Result::<_, rusqlite::Error>::Ok(updated)
        })
        .await?;

    sessions.event(SessionEvent::MessageEdited { chat, message }).await;
    Ok(Status::Accepted)
}

#[post("/view_message/<message>")]
async fn view_message(db: Db, user: UserLoggedIn, message: MessageId, sessions: &State<SessionManager>) -> Result<Status> {
    
    let chat = db
        .run(move |db| {
            db.query_row(
                "
                UPDATE messages
                SET views=IFNULL(views+1, 1)
                WHERE message_id=:message_id
                AND chat_id IN (SELECT chat_id FROM chat_members WHERE member_id=:user_id)
                AND TRUE=(SELECT track_views FROM chats WHERE messages.chat_id=chats.chat_id)
                RETURNING chat_id
            ",
                named_params![
                    ":user_id": user, 
                    ":message_id": message],
            |row| row.get(0)
            )
        })
        .await?;
    sessions.event(SessionEvent::MessageEdited { chat, message }).await;
    Ok(Status::Accepted)
}



#[post("/set_message_pinned/<message>", data = "<pinned>")]
async fn set_message_pinned(
    db: Db,
    user: UserLoggedIn,
    message: MessageId,
    pinned: Json<bool>,
    sessions: &State<SessionManager>
) -> Result<Status> {
    let chat = db
        .run(move |db| {
            db.query_row(
                "
                UPDATE messages
                SET pinned=?3
                WHERE message_id=?2 AND (
                    (SELECT privilage FROM chat_members WHERE chat_members.member_id=?1 AND chat_members.chat_id=messages.chat_id) 
                    >= 
                    (SELECT sending_privilage FROM chats WHERE chats.chat_id=messages.chat_id)
                )
                RETURNING chat_id
            ",
                params![user, message, pinned.0],
                |row| row.get(0)
            )
        })
        .await?;
    sessions.event(SessionEvent::MessageEdited { chat, message }).await;

    Ok(Status::Accepted)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    message_id: i64,
    message: String,
    reply_to: Option<i64>,
    posted: i64,
    last_edited: Option<i64>,
    sender_id: Option<UserId>,
    views: Option<i64>,
    pinned: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct GetMessages {
    previous: Option<usize>,
    limit: Option<usize>,
}

#[post("/get_messages/<chat>", data = "<get>")]
async fn get_messages(
    db: Db,
    user: UserLoggedIn,
    chat: ChatId,
    get: Json<GetMessages>,
) -> Result<Json<Vec<Message>>> {
    let messages = db
        .run(move |conn| {
            conn.prepare(
                "
        SELECT 
            message_id, message, reply_to, posted, last_edited, sender_id, views, pinned
        FROM 
            messages
        WHERE
            chat_id=:chat_id
            AND
            :user_id IN (SELECT member_id FROM chat_members WHERE chat_id=:chat_id)
        ORDER BY message_id DESC, posted DESC
        LIMIT :limit OFFSET :offset",
            )?
            .query_map(
                named_params! {
                    ":limit": get.limit.unwrap_or(50),
                    ":offset": get.previous.unwrap_or(0),
                    ":chat_id": chat,
                    ":user_id": user,
                },
                |row| {
                    Ok(Message {
                        message_id: row.get(0)?,
                        message: row.get(1)?,
                        reply_to: row.get(2)?,
                        posted: row.get(3)?,
                        last_edited: row.get(4)?,
                        sender_id: row.get(5)?,
                        views: row.get(6)?,
                        pinned: row.get(7)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()
        })
        .await?;

    Ok(Json(messages))
}

#[get("/get_message/<message_id>")]
async fn get_message(
    db: Db,
    user: UserLoggedIn,
    message_id: i64,
) -> Result<Json<Message>> {
    let messages = db
        .run(move |conn| {
            conn.query_row(
                "
        SELECT 
            message_id, message, reply_to, posted, last_edited, sender_id, views, pinned
        FROM 
            messages
        WHERE
            message_id=:message_id
            AND
            :user_id IN (SELECT member_id FROM chat_members WHERE chat_members.chat_id=messages.chat_id)",
                named_params! {
                    ":message_id": message_id,
                    ":user_id": user,
                },
                |row| {
                    Ok(Message {
                        message_id: row.get(0)?,
                        message: row.get(1)?,
                        reply_to: row.get(2)?,
                        posted: row.get(3)?,
                        last_edited: row.get(4)?,
                        sender_id: row.get(5)?,
                        views: row.get(6)?,
                        pinned: row.get(7)?,
                    })
                },
            )
        })
        .await?;

    Ok(Json(messages))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        send_message,
        delete_message,
        update_message,
        view_message,
        set_message_pinned,
        get_messages,
        get_message
    ]
}
