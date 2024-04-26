use crate::database::Db;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes, Route};

use rocket::{delete, http::Status};
use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use super::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Contact {
    #[serde(skip_deserializing)]
    pub user_id: i64,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_file_id: Option<i64>,
}

#[get("/get_contacts")]
async fn list_users(db: Db, user: users::UserId) -> Result<Json<Vec<Contact>>> {
    let ids = db
        .run(move |conn| {
            conn.prepare(
                "
            SELECT 
                user.user_id, user.username user.phone_number user.bio, user.pfp_file_id 
            FROM 
                users
            INNER JOIN contacts ON contacts.contact_user_id = users.user_id
            WHERE user_id=:user_id
            ",
            )?
            .query_map(
                named_params![
                    ":user_id": user.0
                ],
                |row| {
                    Ok(Contact {
                        user_id: row.get(0)?,
                        display_name: match row.get::<_, Option<String>>(1)? {
                            Some(name) => name,
                            None => row.get(2)?,
                        },
                        bio: row.get(3)?,
                        pfp_file_id: row.get(4)?,
                    })
                },
            )?
            .collect::<Result<Vec<Contact>, _>>()
        })
        .await?;

    Ok(Json(ids))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ContactId {
    contact_id: i64,
}

#[get("/add_contact", data = "<contact>")]
async fn add_contact(db: Db, user: users::UserId, contact: Json<ContactId>) -> Result<Status> {
    let affected =
        db.0.run(move |db| {
            db.execute(
                "
        INSERT INTO contacts
            (user_id, contact_user_id)
        VALUES
            (:user_id, :contact_id)",
                named_params![
                    ":user_id": user.0,
                    ":contact_id": contact.contact_id
                ],
            )
        })
        .await?;

    match affected {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[delete("/delete_contact", data = "<contact>")]
async fn delete_contact(db: Db, user: users::UserId, contact: Json<ContactId>) -> Result<Status> {
    let affected =
        db.0.run(move |db| {
            db.execute(
                "
        DELETE FROM contacts
        WHERE
            user_id=:user_id AND contact_user_id=:contact_id
        ",
                named_params![
                    ":user_id": user.0,
                    ":contact_id": contact.contact_id
                ],
            )
        })
        .await?;

    match affected {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[post("/find_user_email", data = "<email>")]
async fn find_user_email(db: Db, _user: users::UserId, email: String) -> Result<Json<Option<i64>>> {
    let res =
        db.0.run(move |db| {
            db.query_row(
                "
        SELECT user_id FROM users
        WHERE
            email=:email
        ",
                named_params![
                    ":email": email
                ],
                |row| row.get::<_, i64>(0),
            )
        })
        .await;
    match res {
        Ok(id) => Ok(Some(id).into()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None.into()),
        Err(err) => Err(err.into()),
    }
}

#[post("/find_user_phone", data = "<phone>")]
async fn find_user_phone(db: Db, _user: users::UserId, phone: String) -> Result<Json<Option<i64>>> {
    let res =
        db.0.run(move |db| {
            db.query_row(
                "
        SELECT user_id FROM users
        WHERE
            phone_number=:phone
        ",
                named_params![
                    ":phone": phone
                ],
                |row| row.get::<_, i64>(0),
            )
        })
        .await;
    match res {
        Ok(id) => Ok(Some(id).into()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None.into()),
        Err(err) => Err(err.into()),
    }
}

#[post("/find_user_username", data = "<username>")]
async fn find_user_username(
    db: Db,
    _user: users::UserId,
    username: String,
) -> Result<Json<Option<i64>>> {
    let res =
        db.0.run(move |db| {
            db.query_row(
                "
        SELECT user_id FROM users
        WHERE
            username=:username
        ",
                named_params![
                    ":username": username
                ],
                |row| row.get::<_, i64>(0),
            )
        })
        .await;
    match res {
        Ok(id) => Ok(Some(id).into()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None.into()),
        Err(err) => Err(err.into()),
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        list_users,
        add_contact,
        delete_contact,
        find_user_email,
        find_user_username,
        find_user_phone,
    ]
}
