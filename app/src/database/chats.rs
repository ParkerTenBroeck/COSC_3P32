use std::time::SystemTime;

use crate::database::Db;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes};

use rocket_sync_db_pools::rusqlite;

use self::rusqlite::params;
use rocket::{delete, http::Status};

use super::*;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct CreateDM {
    other: i64,
}

#[post("/create_dm", data = "<dm>")]
async fn create_dm(db: Db, user: users::UserId, dm: Json<CreateDM>) -> Result<Json<i64>> {
    let val = db.run(move |db|{
            let tran = db.transaction()?;

            let chat_id = tran.query_row("
            INSERT INTO chats
                (primary_owner, secondary_owner, sending_privilage, track_views, max_members)
            SELECT
                ?1, ?2, 0, FALSE, 2
            WHERE NOT EXISTS 
                (SELECT 1 FROM chats WHERE (primary_owner=?1 AND secondary_owner=?2) OR (primary_owner=?2 AND secondary_owner=?1))
            RETURNING chat_id", params![user.0, dm.other], |row| row.get(0))?;

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
async fn create_group(db: Db, user: users::UserId, group: Json<CreateGroup>) -> Result<Json<i64>> {
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
                |row| row.get(0),
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
    user: users::UserId,
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
                ?1, 128, TRUE, 9223372036854775807, ?2
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
                |row| row.get(0),
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


#[get("/join_chat/<_chat_id>", rank = 2)]
async fn join_chat_user_logedout(_chat_id: i64) -> &'static str {
    "You must be logged in to join a chat!"
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ChatId {
    pub chat_id: i64,
}

#[get("/join_chat/<chat_id>")]
async fn join_chat_user(db: Db, user: users::UserId, chat_id: i64) -> Result<Result<Redirect, &'static str>> {
    let affected =
        db.0.run(move |db| {
            db.execute(
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
                params![chat_id, user.0],
            )
        })
        .await?;

    match affected {
        1 => Ok(Ok(Redirect::to("/"))),
        _ => Ok(Err("Cannot join group, Invalid or max chat limit reached")),
    }
}

#[post("/join_chat", data = "<chat>")]
async fn join_chat(db: Db, user: users::UserId, chat: Json<ChatId>) -> Result<Status> {
    let affected =
        db.0.run(move |db| {
            db.execute(
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
            )
        })
        .await?;

    match affected {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[post("/leave_chat", data = "<chat>")]
async fn leave_chat(db: Db, user: users::UserId, chat: Json<ChatId>) -> Result<Status> {
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
async fn delete_chat(db: Db, user: users::UserId, chat: Json<ChatId>) -> Result<Status> {
    let affected =
        db.0.run(move |db| {
            db.execute(
                "
            DELETE FROM chats
                WHERE
                chat_id = ?2 AND (primary_owner=?1 OR IFNULL(secondary_owner=?1, FALSE))
            ",
                params![user.0, chat.chat_id],
            )
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
    user: users::UserId,
    updated: Json<UpdatePerm>,
) -> Result<Status> {
    let affected = db.0.run(move |db|{
            db.execute("
            UPDATE chat_members
                SET privilage=?3
            WHERE
                chat_id=?1 AND member_id=?2 
                AND ?3<(SELECT SUM(chat_members) FROM chat_members WHERE chat_id=?1 AND member_id=?4)
                AND privilage<(SELECT SUM(chat_members) FROM chat_members WHERE chat_id=?1 AND member_id=?4)
            ", params![updated.chat_id, updated.user_id, updated.new_perm, user.0])
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
async fn list_chats(db: Db, user: users::UserId) -> Result<Json<Vec<Chat>>> {
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub(super) struct ChatMember {
    user_id: i64,
    privilage: u8,
}

#[post("/list_chat_members", data = "<chat>")]
async fn list_chat_members(
    db: Db,
    user: users::UserId,
    chat: Json<ChatId>,
) -> Result<Json<Vec<ChatMember>>> {
    Ok(db
        .run(move |db| {
            db.prepare(
                "
            SELECT member_id, privilage
            FROM chat_members
            WHERE ?1=(SELECT member_id FROM chat_members WHERE chat_id=?2 AND member_id=?1)
                    AND chat_id=?2
                    ORDER BY privilage DESC
            ",
            )?
            .query_map(params![user.0, chat.chat_id], |row| {
                Ok(ChatMember {
                    user_id: row.get(0)?,
                    privilage: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .await?
        .into())
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct UpdateChatNotif {
    chat_id: i64,
    notifications: bool,
}

#[post("/update_chat_notifications", data = "<update>")]
async fn update_chat_notifications(
    db: Db,
    user: users::UserId,
    update: Json<UpdateChatNotif>,
) -> Result<Status> {
    let updated = db
        .run(move |db| {
            db.execute(
                "
                UPDATE chat_members
                SET wants_notifications=?3
                WHERE chat_id=?2 AND member_id=?1
            ",
                params![user.0, update.chat_id, update.notifications],
            )
        })
        .await?;
    match updated {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[post("/mark_chat_read", data = "<update>")]
async fn mark_chat_read(db: Db, user: users::UserId, update: Json<ChatId>) -> Result<Status> {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let updated = db
        .run(move |db| {
            db.execute(
                "
                UPDATE chat_members
                SET last_seen=?3
                WHERE chat_id=?2 AND member_id=?1
            ",
                params![user.0, update.chat_id, since_the_epoch as i64],
            )
        })
        .await?;
    match updated {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

pub fn user_routes() -> Vec<rocket::Route>{
    routes![
        join_chat_user,
        join_chat_user_logedout
    ]
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
        list_chat_members,
        update_chat_notifications,
        mark_chat_read
    ]
}
