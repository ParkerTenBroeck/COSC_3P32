use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, routes, State};

use rocket_sync_db_pools::rusqlite;

use super::*;

#[get("/chat_events/<chat_id>")]
fn chat_events(shutdown: rocket::Shutdown, listeners: &State<ChatEvents>, chat_id: i64) -> EventStream![Event + '_] {
    EventStream! {
        let mut channel_updates = listeners.listen(chat_id).await;
        let mut shutdown = std::pin::pin!(shutdown);
        
        loop {
            rocket::tokio::select! {
                _ = shutdown.as_mut() => {break}
                val = channel_updates.recv() => {
                    if let Some(val) = val{
                        yield Event::json(&val);
                    }else{
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "tag", content = "id")]
pub enum ChatEvent {
    NewMessage(i64),
    MessageEdited(i64),
    MessageDeleted(i64),
    
    UserUpdated(i64),
    UserJoined(i64),
    UserLeft(i64),
}
pub type ChatEvents = EventThing<ChatEvent>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "tag", content = "id")]
pub enum UserEvent {
    NewChat(i64),
    DeletedChat(i64),
    ChatNotification(i64)
}

pub type UserEvents = EventThing<UserEvent>;


#[derive(Debug)]
pub struct EventThing<T: Send + Clone>{
    map: rocket::tokio::sync::Mutex<HashMap<i64, Vec<rocket::tokio::sync::mpsc::Sender<T>>>>
}

impl<T: Send + Clone> std::default::Default for EventThing<T>{
    fn default() -> Self {
        Self { map: Default::default() }
    }
}

impl<T: Send + Clone> EventThing<T>{
    pub async fn event(&self, channel_id: i64, event: T) {
        let mut guard = self.map.lock().await;
        if let Some(some) = guard.get_mut(&channel_id) {
            

            some.retain_mut(|s|{
                !s.is_closed()
            });

            for s in some.iter_mut(){
                _ = s.send(event.clone()).await;
            }
            // rocket::tokio::join!()
            if some.len() == 0{
                guard.remove(&channel_id);
            }else{
                some.shrink_to_fit()
            }
        }
    }

    pub async fn listen(&self, channel_id: i64) -> rocket::tokio::sync::mpsc::Receiver<T>{
        let (sender, receiver) = rocket::tokio::sync::mpsc::channel(10);
        self.map.lock().await.entry(channel_id).or_default().push(sender);
        receiver
    }
}


pub fn adhoc() -> AdHoc {
    AdHoc::on_ignite("events", |rocket| async {
        rocket.mount(
            "/database",
            routes![
                chat_events
            ],
        )
        .manage(ChatEvents::default())
        .manage(UserEvents::default())
    })
}
