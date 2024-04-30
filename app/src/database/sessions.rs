use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use rocket::serde::json::Json;
use rocket::tokio::sync::mpsc::{channel, Receiver, Sender};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::RwLock;
use rocket::{get, post, routes, Request, State};

use rocket_sync_db_pools::rusqlite;
use rusqlite::named_params;

use self::chats::ChatId;
use self::messages::MessageId;
use self::users::{UserId, UserLoggedIn};

use super::*;

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
async fn login(db: Db, jar: &CookieJar<'_>, sessions: &State<SessionManager>, login: Json<LoginRequest>) -> Result<Json<AuthResult>> {
    jar.remove_private("user_id");

    let id: Result<i64, _> = db
        .run(move |conn| {
            conn.query_row(
                "SELECT user_id FROM users WHERE email=:email AND password=:password",
                named_params![
                    ":email": login.email,
                    ":password": login.password],
                |r| r.get(0),
            )
        })
        .await;

    match id {
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AuthResult::Failed.into()),
        Ok(id) => {
            jar.add_private(Cookie::build(("user_id", format!("{id}"))));
            jar.add_private(("session_id", format!("{}", sessions.next_session_id().0)));
            Ok(AuthResult::Authorized.into())
        }
        Err(err) => Err(err.into()),
    }
}

#[post("/logout")]
async fn logout(jar: &CookieJar<'_>) {
    jar.remove_private("user_id");
    jar.remove_private("session_id");
}


#[post("/session_subscribe_to_user/<user>")]
async fn session_subscribe_to_user(db: Db, guard: SessionGuard, user: UserId, sessions: &State<SessionManager>) -> Result<Status>{
    // let res: bool = db.run(move |db|{
    //     db.query_row("
    //         :other_id IN (SELECT member_id FROM chat_members WHERE chat_id IN (SELECT chat_id FROM chat_members WHERE member_id=:user_id))
    //         OR :other_id IN (SELECT contact_user_id FROM contacts WHERE user_id=:user_id)
    //     ", named_params! [
    //         ":user_id": guard.1.id(),
    //         ":other_id": user
    //     ],
    //     |row| row.get(0))
    // }).await?;

    // if !res{
    //     return Ok(Status::NotAcceptable)
    // }
    sessions.subsribe_to_user(guard, user).await;
    
    Ok(Status::Accepted)
}

#[post("/session_unsubscribe_from_user/<user>")]
async fn session_unsubscribe_from_user(guard: SessionGuard,  user: UserId, sessions: &State<SessionManager>) -> Result<Status>{
    sessions.unsubscribe_from_user(guard, user).await;
    Ok(Status::Accepted)
}

#[post("/session_unsubscribe_chat")]
async fn session_unsubscribe_chat(guard: SessionGuard,  sessions: &State<SessionManager>) -> Result<Status>{
    sessions.unsubscribe_from_chat(guard).await;
    Ok(Status::Accepted)
}

#[post("/session_subscribe_to_chat/<chat>")]
async fn session_subscribe_to_chat(db: Db, guard: SessionGuard, chat: ChatId, sessions: &State<SessionManager>) -> Result<Status>{
    let res: UserId = db.run(move |db|{
        db.query_row("
            SELECT member_id FROM chat_members WHERE member_id=:user_id AND chat_id=:chat_id
        ", named_params! [
            ":user_id": guard.1.id(),
            ":chat_id": chat
        ],
        |row| row.get(0))
    }).await?;

    if res != guard.1.id(){
        return Ok(Status::NotAcceptable)
    }
    sessions.subsribe_to_chat(guard, chat).await;
    
    Ok(Status::Accepted)
}

#[get("/open_session")]
fn open_session<'a>(db: Db, shutdown: rocket::Shutdown, guard: SessionGuard, sessions: &'a State<SessionManager>) -> EventStream![Event + 'a] {

    EventStream! {

        _ = db.run(move |db|{
            db.execute("UPDATE users SET availability=1 WHERE user_id=:user_id", named_params! [":user_id": guard.1])
        }).await;
        drop(db);
       
        let (stream_id, mut receiver) = sessions.begin(guard).await;
        let mut shutdown = std::pin::pin!(shutdown);

        struct Bruh(SessionGuard, u64, SessionManager);
        let _bruh = Bruh(guard, stream_id, (*sessions).clone());
        impl Drop for Bruh{
            fn drop(&mut self){
                let guard = self.0;
                let sessions = self.2.clone();
                let stream_id = self.1;
                rocket::tokio::spawn(async move {
                    sessions.remove_active(stream_id, guard).await;
                });
            }
        }
        
        loop {
            rocket::tokio::select! {
                _ = shutdown.as_mut() => {break}
                val = receiver.recv() => {
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



pub fn adhoc() -> AdHoc {
    AdHoc::on_ignite("sessions", |rocket| async {
        rocket.mount(
            "/database",
            routes![
                open_session,
                session_subscribe_to_user,
                session_subscribe_to_chat,
                session_unsubscribe_chat,
                session_unsubscribe_from_user,
                login,
                logout
            ],
        )
        .manage(SessionManager::default())
    })
}

#[derive(Debug, Clone, Copy, std::hash::Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId(u64);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private("session_id")
            .and_then(|c| c.value().parse().ok())
            .map(SessionId)
            .or_forward(Status::Unauthorized)
    }
}

#[derive(Debug, Clone, Copy)]
struct SessionGuard(SessionId, UserLoggedIn);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionGuard {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        UserLoggedIn::from_request(request).await.and_then(|user|{
            request
                .cookies()
                .get_private("session_id")
                .and_then(|c| c.value().parse().ok())
                .map(SessionId)
                .map(|id| SessionGuard(id, user))
                .or_forward(Status::Unauthorized)
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(tag = "tag")]
pub enum SessionEvent{
    NewMessage{chat: ChatId, message: MessageId},
    MessageEdited{chat: ChatId, message: MessageId},
    MessageDeleted{chat: ChatId, message: MessageId},
    
    UserJoined{chat: ChatId, user: UserId},
    UserLeft{chat: ChatId, user: UserId},

    UserUpdated{user: UserId},
    UserDeleted{user: UserId},

    WhoAmI{user: UserId},
}

#[derive(Debug)]
struct SSender(Sender<SessionEvent>);

impl Deref for SSender{
    type Target = Sender<SessionEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SSender{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq for SSender{
    fn eq(&self, other: &Self) -> bool {
        self.0.same_channel(&other.0)
    }
}

impl Eq for SSender{ }

impl Hash for SSender{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // this is bad
        let v: &Sender<_> = &self.0;
        let v: & *const () = unsafe {std::mem::transmute(v)};
        (*v).hash(state)
    }
}

#[test]
fn bruh(){

    let s: SSender = channel::<SessionEvent>(10).0.into();
    s.hash(&mut std::hash::DefaultHasher::new())
}

impl Into<SSender> for Sender<SessionEvent>{
    fn into(self) -> SSender {
        SSender(self)
    }
}

#[derive(Debug)]
struct Listeners<T: Hash + Eq>{
    map: RwLock<HashMap<T, HashSet<SSender>>>
}

impl<T: Hash + Eq> std::default::Default for Listeners<T>{
    fn default() -> Self {
        Self { map: Default::default() }
    }
}

impl<T: Hash + Eq> Listeners<T>{
    async fn event(&self, k: T, event: SessionEvent){
        let guard = self.map.read().await;
        if let Some(senders) = guard.get(&k){
            for sender in senders{
                _ = sender.send(event).await;
            }
        }
    }

    async fn listen(&self, k: T, sender: Sender<SessionEvent>){
        self.map.write().await.entry(k).or_default().insert(sender.into());
    }

    async fn remove(&self, k: T, sender: &Sender<SessionEvent>){
        let mut guard = self.map.write().await;
        if let Some(v) = guard.get_mut(&k){
            v.remove(&sender.clone().into());
            if v.is_empty(){
                guard.remove(&k);
            }
        }
    }
}

#[derive(Debug)]
struct ActiveSession{
    sender: Sender<SessionEvent>,
    self_listener: UserId,
    stream_id: u64,
    chat_listener: Option<ChatId>,
    other_listeners: HashSet<UserId>,
}

impl ActiveSession{
    pub fn new(guard: SessionGuard, stream_id: u64, sender: Sender<SessionEvent>) -> Self{
        Self { stream_id, sender, chat_listener: None, self_listener: guard.1.id(), other_listeners: Default::default() }
    }
}

pub type SessionManager = Arc<SessionManagerInner>;

#[derive(Debug, Default)]
pub struct SessionManagerInner{
    next_session_id: AtomicU64,
    next_stream_id: AtomicU64,
    active_sessions: RwLock<HashMap<SessionId, ActiveSession>>,


    self_listeners: Listeners<UserId>,
    other_listeners: Listeners<UserId>,
    chat_listeners: Listeners<ChatId>,
}

impl SessionManagerInner{
    #[inline(always)]
    pub async fn event(&self, event: SessionEvent){
        match event{
            SessionEvent::NewMessage { chat, .. }
            | SessionEvent::MessageEdited { chat, .. }
            | SessionEvent::MessageDeleted { chat, .. } => {
                self.chat_listeners.event(chat, event).await
            },
            
            SessionEvent::UserJoined { chat, user } 
            | SessionEvent::UserLeft { chat, user } => {
                self.chat_listeners.event(chat, event).await;
                self.self_listeners.event(user, event).await;
            },

            SessionEvent::UserUpdated { user } => {
                self.other_listeners.event(user, event).await;
                self.self_listeners.event(user, event).await;
            },

            SessionEvent::WhoAmI { user } 
            | SessionEvent::UserDeleted { user } => {
                self.self_listeners.event(user, event).await;
            },
        }
    }

    fn next_session_id(&self) -> SessionId{
        SessionId(self.next_session_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }

    fn next_stream_id(&self) -> u64{
        self.next_stream_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    async fn begin(&self, session: SessionGuard) -> (u64, Receiver<SessionEvent>){
        let(sender, receiver) = channel(10);
        
        let stream_id = self.next_stream_id();
        let old = self.active_sessions.write().await.insert(session.0, ActiveSession::new(session, stream_id,  sender.clone()));
        if let Some(old) = old{
            self.remove_session(old).await;
        }
        _ = sender.send(SessionEvent::WhoAmI { user: session.1.id() }).await;
        self.self_listeners.listen(session.1.id(), sender).await;
        

        (stream_id, receiver)
    }

    async fn unsubscribe_from_chat(&self, guard: SessionGuard){
        let mut lock = self.active_sessions.write().await;
        if let Some(active) = lock.get_mut(&guard.0){
            if let Some(current) = active.chat_listener.take(){
                self.chat_listeners.remove(current, &active.sender).await;
            }
        }
    }

    async fn subsribe_to_chat(&self, guard: SessionGuard, chat: ChatId){
        let mut lock = self.active_sessions.write().await;
        if let Some(active) = lock.get_mut(&guard.0){
            if let Some(current) = active.chat_listener.take(){
                self.chat_listeners.remove(current, &active.sender).await;
            }

            active.chat_listener = Some(chat);
            self.chat_listeners.listen(chat, active.sender.clone()).await;
        }
    }

    async fn unsubscribe_from_user(&self, guard: SessionGuard, user: UserId){
        let mut lock = self.active_sessions.write().await;
        if let Some(active) = lock.get_mut(&guard.0){
            if active.other_listeners.remove(&user){
                self.other_listeners.remove(user, &active.sender).await;
            }
        }
    }

    async fn subsribe_to_user(&self, guard: SessionGuard, user: UserId){
        let mut lock = self.active_sessions.write().await;
        if let Some(active) = lock.get_mut(&guard.0){
            if active.other_listeners.insert(user){
                self.other_listeners.listen(user, active.sender.clone()).await;
            }
        }
    }

    async fn remove_active(&self, stream_id: u64, guard: SessionGuard){
        let mut lock = self.active_sessions.write().await;
        let entry = lock.entry(guard.0);
        match entry{
            std::collections::hash_map::Entry::Occupied(v) => {
                if v.get().stream_id == stream_id{
                    self.remove_session(v.remove()).await;
                }
            },
            std::collections::hash_map::Entry::Vacant(_) => {},
        }
    }

    async fn remove_session(&self, session: ActiveSession){
        self.self_listeners.remove(session.self_listener, &session.sender).await;
        if let Some(chat) = session.chat_listener{
            self.chat_listeners.remove(chat, &session.sender).await;
        }
        for user in session.other_listeners.iter(){
            self.other_listeners.remove(*user, &session.sender).await;
        }
    }
}