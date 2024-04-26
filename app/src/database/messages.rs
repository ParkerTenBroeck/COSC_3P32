use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::SystemTime;

use crate::database::Db;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes, State};

use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use self::rusqlite::params;
use rocket::{delete, http::Status};

use super::*;

#[derive(Debug, Default)]
struct MessageChannelUpdates {
    members: AtomicUsize,
    con: rocket::tokio::sync::Notify,
    queue: rocket::tokio::sync::Mutex<VecDeque<(usize, MessageUpdate)>>,
}

struct MessageChannelUpdatesGuard {
    inner: Arc<MessageChannelUpdates>,
}

impl Drop for MessageChannelUpdatesGuard {
    fn drop(&mut self) {
        self.inner.leave()
    }
}

impl MessageChannelUpdatesGuard {
    pub async fn next(&self) -> MessageUpdate {
        loop {
            let mut guard = self.inner.queue.lock().await;

            match guard.get_mut(0) {
                Some((1|0, _)) => return guard.pop_front().unwrap().1,
                Some((number, message)) => {
                    *number -= 1;
                    return message.clone();
                }
                None => {
                    drop(guard);
                    self.inner.con.notified().await;
                }
            }
        }
    }
}

impl MessageChannelUpdates {
    pub fn join(self: Arc<Self>) -> MessageChannelUpdatesGuard {
        self.members
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        MessageChannelUpdatesGuard { inner: self }
    }
    fn leave(&self) {
        self.members
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
    async fn push(&self, update: MessageUpdate) {
        let num = self.members.load(std::sync::atomic::Ordering::Acquire);
        if num > 0{
            self.queue.lock().await.push_back((
                num,
                update,
            ));
            self.con.notify_waiters();
        }
    }
}

#[derive(Debug, Default)]
struct MessageUpdates {
    items: rocket::tokio::sync::Mutex<HashMap<i64, Arc<MessageChannelUpdates>>>,
}

impl MessageUpdates {
    pub async fn join(&self, channel_id: i64) -> MessageChannelUpdatesGuard {
        self.items.lock().await.entry(channel_id).or_default().clone().join()
    }

    pub async fn push(&self, channel_id: i64, update: MessageUpdate) {
        self.items
            .lock()
            .await
            .entry(channel_id)
            .or_default()
            .push(update)
            .await
    }
}

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
    listeners: &State<MessageUpdates>
) -> Result<Json<i64>> {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let chat_id = message.chat_id;

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

    listeners.push(chat_id, MessageUpdate::New(mid)).await;

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
    listeners: &State<MessageUpdates>
) -> Result<Status> {
    let message_id = message_id.message_id;
    let chat_id: i64 = db
        .run(move |conn| {
            conn.query_row(
                "
        DELETE FROM messages 
        WHERE message_id=:message_id AND sender_id=:user_id
        RETURNING chat_id",
                named_params![
                    ":message_id": message_id, 
                    ":user_id": user.0],
            |row| row.get(0))
        })
        .await?;

    listeners.push(chat_id, MessageUpdate::Deleted(message_id)).await;
    Ok(Status::Accepted)
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
    listeners: &State<MessageUpdates>
) -> Result<Status> {
    let message_id = message.message_id;
    let chat_id = db
        .run(move |conn| {
            let since_the_epoch = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let tran = conn.transaction()?;
            let updated: i64 = tran.query_row(
                "
            UPDATE messages
            SET message = ?3, last_edited=?4
            WHERE message_id=?1 AND sender_id=?2
            RETURNING chat_id",
                params![
                    message_id,
                    user.0,
                    message.message,
                    since_the_epoch as i64
                ], |row| row.get(0)
            )?;
            
            tran.commit()?;
            Result::<_, rusqlite::Error>::Ok(updated)
        })
        .await?;

        listeners.push(chat_id, MessageUpdate::Updated(message_id)).await;
    Ok(Status::Accepted)
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct UpdateMessagePinned {
    message_id: i64,
    pinned: bool,
}

#[post("/set_message_pinned", data = "<update>")]
async fn set_message_pinned(
    db: Db,
    user: users::UserId,
    update: Json<UpdateMessagePinned>,
) -> Result<Status> {
    let updated = db
        .run(move |db| {
            db.execute(
                "
                UPDATE messages
                SET pinned=?3
                WHERE message_id=?2 AND (
                    (SELECT privilage FROM chat_members WHERE chat_members.member_id=?1 AND chat_members.chat_id=messages.chat_id) 
                    >= 
                    (SELECT sending_privilage FROM chats WHERE chats.chat_id=messages.chat_id)
                )
            ",
                params![user.0, update.message_id, update.pinned],
            )
        })
        .await?;
    // listeners.push(update., update)
    match updated {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
enum MessageUpdate {
    New(i64),
    Updated(i64),
    Pinned(i64),
    Deleted(i64),
}

#[get("/listen_for_messages/<chat_id>")]
fn listen_for_messages(shutdown: rocket::Shutdown, listeners: &State<MessageUpdates>, chat_id: i64) -> EventStream![Event + '_] {
    EventStream! {
        let channel_updates = listeners.join(chat_id).await;
        let mut shutdown = std::pin::pin!(shutdown);
        
        loop {
            rocket::tokio::select! {
                _ = shutdown.as_mut() => {break}
                val = channel_updates.next() => {
                    yield Event::json(&val);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    message_id: i64,
    message: String,
    reply_to: Option<i64>,
    posted: i64,
    last_edited: Option<i64>,
    sender: i64,
    views: Option<i64>,
    pinned: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct GetMessages {
    chat_id: i64,
    previous: Option<usize>,
    limit: Option<usize>,
}

#[post("/get_messages", data = "<get>")]
async fn get_messages(
    db: Db,
    user: users::UserId,
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
        ORDER BY posted ASC, message_id ASC
        LIMIT :limit OFFSET :offset",
            )?
            .query_map(
                named_params! {
                    ":limit": get.limit.unwrap_or(50),
                    ":offset": get.previous.unwrap_or(0),
                    ":chat_id": get.chat_id,
                    ":user_id": user.0,
                },
                |row| {
                    Ok(Message {
                        message_id: row.get(0)?,
                        message: row.get(1)?,
                        reply_to: row.get(2)?,
                        posted: row.get(3)?,
                        last_edited: row.get(4)?,
                        sender: row.get(5)?,
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

pub fn adhoc() -> AdHoc {
    AdHoc::on_ignite("messages", |rocket| async {
        rocket.mount(
            "/database",
            routes![
                send_message,
                delete_message,
                update_message,
                view_message,
                set_message_pinned,
                get_messages,
                listen_for_messages
            ],
        ).manage(MessageUpdates::default())
    })
}
