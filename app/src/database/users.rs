use crate::database::Db;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes};

use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use self::rusqlite::params;

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
            .map(UserId)
            .or_forward(Status::Unauthorized)
    }
}

impl UserId {
    pub async fn load(self, conn: &Db) -> Result<User, rusqlite::Error> {
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
        user_id.load(&conn).await.or_forward(Status::Unauthorized)
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
                (phone_number, name, email, location, username, password) 
            VALUES 
                (?1, ?2, ?3, ?4, ?5, ?6)
            RETURNING user_id",
                params![
                    create_user.phone_number,
                    create_user.name,
                    create_user.email,
                    create_user.location,
                    create_user.username,
                    create_user.password,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct UpdateUser {
    phone_number: String,
    name: String,
    email: String,
    location: String,
    username: String,
    password: String,
    bio: String,
    availability: u8,
    pfp_file_id: Option<i64>,
}

#[post("/update_user", data = "<updates>")]
async fn update_user(db: Db, user: UserId, updates: Json<UpdateUser>) -> Result<Status> {
    let affected = db
        .run(move |db| {
            Result::<_, rusqlite::Error>::Ok(db.execute(
                "
            UPDATE users
            SET phone_number=:phone_number, name=:name, email=:email, location=:location, username=:username, password=:password, bio=:bio, pfp_file_id=:pfp_file_id
            WHERE user_id=?1
        ",
                named_params![
                    ":user_id": user.0,
                    ":phone_number": updates.phone_number,
                    ":name": updates.name,
                    ":email": updates.email,
                    ":location": updates.location,
                    ":username": updates.username,
                    ":password": updates.password,
                    ":bio": updates.bio,
                    ":pfp_file_id": updates.pfp_file_id,
                ],
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
async fn login(db: Db, jar: &CookieJar<'_>, login: Json<LoginRequest>) -> Result<Json<AuthResult>> {
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
    pub user_id: i64,
    pub phone_number: String,
    pub name: String,
    pub email: String,
    pub location: String,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub php_file_id: Option<i64>,
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

#[get("/get_username/<user_id>")]
async fn get_username(db: Db, _user: users::UserId, user_id: i64) -> Result<String>{
    let username = db
        .run(move |conn| {
            conn.prepare(
                "
            SELECT 
                username phone_number
            FROM 
                users
            WHERE user_id=:user_id
            ",
            )?
            .query_map(
                named_params![
                    ":user_id": user_id
                ],
                |row| {
                    Ok(match row.get::<_, Option<String>>(0)? {
                        Some(name) => name,
                        None => row.get(1)?,
                    })
                },
            )?
            .collect::<Result<String, _>>()
        })
        .await?;

    Ok(username)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SafeUser {
    #[serde(skip_deserializing)]
    pub user_id: i64,
    pub display_name: String,
    pub bio: String,
    pub pfp_file_id: Option<i64>,
}

#[get("/get_user/<user_id>")]
async fn get_user(db: Db, _user: users::UserId, user_id: i64) -> Result<Json<SafeUser>> {
    let user = db
        .run(move |conn| {
            conn.query_row(
                "
            SELECT 
                username, phone_number, bio, pfp_file_id 
            FROM 
                users
            WHERE user_id=:user_id
            ",
                named_params![
                    ":user_id": user_id
                ],
                |row| {
                    Ok(SafeUser {
                        user_id,
                        display_name: match row.get::<_, Option<String>>(0)? {
                            Some(name) => name,
                            None => row.get(1)?,
                        },
                        bio: row.get(2)?,
                        pfp_file_id: row.get(3)?,
                    })
                },
            )
        })
        .await?;

    Ok(Json(user))
}

pub fn routes() -> Vec<Route> {
    routes![
        new_user,
        login,
        who_am_i,
        logout,
        delete_account,
        update_user,
        get_username,
        get_user
    ]
}
