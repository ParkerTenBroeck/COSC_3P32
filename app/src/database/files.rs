use std::convert::Infallible;

use crate::{database::Db, make_id};
use rocket::{data::{FromData, ToByteUnit}, get, http::Status, outcome::Outcome, post, request::FromRequest, routes, serde::json::Json, Data, Orbit, Request, State};
use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use super::*;


make_id!(FileId);

#[get("/attachments/<id>/<name>")]
async fn attachments_with_name(db: Db, id: FileId, name: String) -> Result<Vec<u8>> {
    Ok(db.run(move |db|{
        db.query_row("
            SELECT contents FROM files WHERE file_id=:id AND file_name=:name
        ", 
        named_params! {
            ":id": id,
            ":name": name
        }, |row| row.get(0))
    }).await?)
}

#[get("/attachments/<id>")]
async fn attachments(db: Db, id: FileId) -> Result<Vec<u8>> {
    Ok(db.run(move |db|{
        db.query_row("
            SELECT contents FROM files WHERE file_id=:id
        ", 
        named_params! {
            ":id": id
        }, |row| row.get(0))
    }).await?)
}
// use rocket::Request;

struct File(Vec<u8>);

struct Req<'a>(&'a Rocket<Orbit>);


#[rocket::async_trait]
impl<'r> FromRequest<'r> for Req<'r> {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Req<'r>, Infallible> {
        rocket::request::Outcome::Success(Req(request.rocket()))
    }
}


#[rocket::async_trait]
impl<'r> FromData<'r> for File {
    type Error =  std::io::Error;

    async fn from_data(_req: &'r Request<'_>, data: Data<'r>) -> rocket::data::Outcome<'r, Self> {
        
        let file = match data.open(10.megabytes()).into_bytes().await{
            Ok(ok) => ok.value,
            Err(e) => return rocket::data::Outcome::Error((Status::NotAcceptable, e)),
        };

        rocket::data::Outcome::Success(File(file))
    }
}


#[post("/upload_file/<name>", data = "<file>")]
async fn upload(file: File, rock: Req<'_>, name: String) -> Result<Result<Json<FileId>, Status>>{

    println!("here: {}", file.0.len());
 
    let id: FileId = Db::get_one(rock.0)
                    .await
                    .expect("database mounted").run(move |db|{
        db.query_row("
            INSERT INTO files (file_name, contents)
            VALUES (:name, :data)
            RETURNING file_id
        ", 
        named_params! {
            ":name": name,
            ":data": file.0
        }, |row| row.get(0))
    }).await?;

    Ok(Ok(id.into()))
}

pub fn routes() -> Vec<rocket::Route>{
    routes![
        attachments_with_name,
        attachments,
        upload
    ]
}