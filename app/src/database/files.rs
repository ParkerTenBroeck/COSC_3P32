use crate::database::Db;
use rocket::{data::ToByteUnit, get, http::Status, post, routes, serde::json::Json, Data};
use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use super::*;

#[get("/attachments/<id>/<name>")]
async fn attachments_with_name(db: Db, id: i64, name: String) -> Result<Vec<u8>> {
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
async fn attachments(db: Db, id: i64) -> Result<Vec<u8>> {
    Ok(db.run(move |db|{
        db.query_row("
            SELECT contents FROM files WHERE file_id=:id
        ", 
        named_params! {
            ":id": id
        }, |row| row.get(0))
    }).await?)
}

#[post("/upload_file/<name>", data = "<file>")]
async fn upload(db: Db, name: String, file: Data<'_>) -> Result<Result<Json<i64>, Status>>{
    
    let file = match file.open(10.gigabytes()).into_bytes().await{
        Ok(ok) => ok.value,
        Err(_) => return Ok(Err(Status::NotAcceptable)),
    };

    let id: i64 = db.run(move |db|{
        db.query_row("
            INSERT INTO files (file_name, contents)
            VALUES (:name, :data)
            RETURNING file_id
        ", 
        named_params! {
            ":name": name,
            ":data": file
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