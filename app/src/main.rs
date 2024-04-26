mod database;

use rocket::{
    fs::{FileServer, Options},
    *,
};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(database::stage_database())
        .mount(
            "/",
            FileServer::new(
                "./static",
                Options::DotFiles | Options::Index | Options::IndexFile,
            )
            .rank(5),
        )
}
