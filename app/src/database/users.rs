use crate::database::Db;
use crate::make_id;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{get, post, routes, State};

use rocket_sync_db_pools::rusqlite;
use rusqlite::{named_params, ToSql};

use self::files::FileId;
use self::rusqlite::params;
use self::sessions::{SessionEvent, SessionManager};

make_id!(UserId);

#[derive(Debug, Clone, Copy, std::hash::Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserLoggedIn(UserId);
impl UserLoggedIn {
    pub fn id(&self) -> UserId {
        self.0
    }
}

impl ToSql for UserLoggedIn{
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}



#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserLoggedIn {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|c| c.value().parse().ok())
            .map(UserId)
            .map(UserLoggedIn)
            .or_forward(Status::Unauthorized)
    }
}

impl UserId {
    pub(super) async fn load(self, conn: &Db) -> Result<User, rusqlite::Error> {
        conn.run(move |db| {
            db.query_row(
                "SELECT user_id, phone_number, name, email, location, username, password, bio, pfp_file_id FROM users WHERE user_id=?1",
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
                        pfp_file_id: row.get(8)?,
                    })
                },
            )
        })
        .await
    }
}

use rocket::{
    delete,
    http::{CookieJar, Status},
    outcome::IntoOutcome,
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
) -> Result<std::result::Result<Created<Json<UserId>>, Status>> {
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
                |r| r.get::<_, UserId>(0),
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
}

#[post("/update_user", data = "<updates>")]
async fn update_user(db: Db, user: UserLoggedIn, updates: Json<UpdateUser>, sessions: &State<SessionManager>) -> Result<Status> {
    let affected = db
        .run(move |db| {
            Result::<_, rusqlite::Error>::Ok(db.execute(
                "
            UPDATE users
            SET phone_number=:phone_number, name=:name, email=:email, location=:location, username=:username, password=:password, bio=:bio
            WHERE user_id=:user_id
        ",
                named_params![
                    ":user_id": user,
                    ":phone_number": updates.phone_number,
                    ":name": updates.name,
                    ":email": updates.email,
                    ":location": updates.location,
                    ":username": updates.username,
                    ":password": updates.password,
                    ":bio": updates.bio,
                ],
            )?)
        })
        .await?;
    sessions.event(SessionEvent::UserUpdated { user: user.id() }).await;

    match affected {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[post("/update_user_pfp/<pfp_id>")]
async fn update_user_pfp(db: Db, user: UserLoggedIn, pfp_id: FileId, sessions: &State<SessionManager>) -> Result<Status> {
    // println!("{pfp_id}");
    let affected = db
        .run(move |db| {
            Result::<_, rusqlite::Error>::Ok(db.execute(
                "
            UPDATE users
            SET pfp_file_id=:pfp_file_id
            WHERE user_id=:user_id
        ",
                named_params![
                    ":user_id": user.0,
                    ":pfp_file_id": pfp_id,
                ],
            )?)
        })
        .await?;

    sessions.event(SessionEvent::UserUpdated { user: user.id() }).await;


    match affected {
        1 => Ok(Status::Accepted),
        _ => Ok(Status::NotAcceptable),
    }
}

#[get("/who_am_i")]
async fn who_am_i(db: Db, user: Option<UserLoggedIn>) -> Result<Json<Option<User>>> {
    if let Some(user) = user{
        Ok(Some(user.0.load(&db).await?).into())
    }else{
        Ok(None.into())
    }
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
    pub pfp_file_id: Option<i64>,
}

#[delete("/delete_account")]
async fn delete_account(db: Db, jar: &CookieJar<'_>, user: UserLoggedIn) -> Result<Status> {
    let affected = db
        .run(move |conn| conn.execute("DELETE FROM users WHERE user_id=:user_id", named_params![":user_id": user]))
        .await?;

    jar.remove_private("user_id");

    match affected {
        0 => Ok(Status::InternalServerError),
        1 => Ok(Status::Accepted),
        _ => Ok(Status::BadRequest),
    }
}

#[get("/get_username/<user_id>")]
async fn get_username(db: Db, _user: UserLoggedIn, user_id: UserId) -> Result<String>{
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
    pub user_id: UserId,
    pub display_name: String,
    pub bio: String,
    pub pfp_file_id: Option<i64>,
}

#[get("/get_user/<user_id>")]
async fn get_user(db: Db, _user: UserLoggedIn, user_id: UserId) -> Result<Json<SafeUser>> {
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
        who_am_i,
        delete_account,
        update_user,
        get_username,
        get_user,
        update_user_pfp
    ]
}
