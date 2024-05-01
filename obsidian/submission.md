## q1

![erd](<ER Diagram.drawio.png>)

the sent by relation does not have total participation as when a user is deleted, the message's sent by field will be set to null indication the message was sent by a deleted user

the chats ISA constraint is not overlaping as a chat can be only a channel, group or direct message but not multiple and it is covering because it must always be one of the three

another thing not modeled is the join links on the groups. this is because we are using the format http://sitename/join_chat/<chat_id> as our join links so links are already stored as the id of the chat

## q2

Files(**<ins>fild_id: INTEGER</ins>**, file_name: TEXT, contents: BLOB)

Users(**<ins>user_id: INTEGER</ins>**, phone_number: TEXT, name: TEXT, email: TEXT, location: TEXT, username: TEXT, password: TEXT, bio: TEXT, availability: INTEGER, pfp_file_id: INTEGER)
pfp_file_id references Files

Contacts(**<ins>user_id: INTEGER, contact_user_id: INTEGER</ins>**)
user_id and contact_user_id reference Users

Messages(**<ins>message_id:INTEGER</ins>**, message: TEXT, reply_to: INTEGER, attachment: INTEGER, timestamp: BIGINT, last_edited: BIGINT, sender_id: INTEGER, chat_id INTEGER, views: INTEGER, pinned: boolean)
sender_id references Users
chat_id references Chats

---------------------------------------------

while there are 3 tables in the ISA relationship with chats, channels, groups, and direct_messages, defined in the erd, we found that we could have their unique fields nullable and differentiate between them by just getting the ones that are not null. This simplifys the queries needed greatly and made it better for our group's concrete implimentation of the assignment for question 6

if there is a secondary owner its a direct message
if there is a sending privlage then its a channel
otherwise its a group

Chats (**<ins>chat_id: INTEGER</ins>**, primary_owner: INTEGER, secondary_owner: INTEGER, sending_privilage: INTEGER, track_views: BOOLEAN, max_members INTEGER, chat_name text)
primary_owner references Users
secondary_owner references Users

---------------------------------------------

Chat_Members (**<ins>chat_id: INTEGER, member_id: INTEGER</ins>**, privilage: INTEGER, wants_notifications: BOOLEAN, last_seen INTEGER,)
chat_id references Chats
member_id references Users

## q3


Files(**<ins>fild_id: INTEGER</ins>**, file_name: TEXT, contents: BLOB)

F(i, n, c)

i -> i, n, c

this is in BCNF as the only fd is the min key

---

Users(**<ins>user_id: INTEGER</ins>**, phone_number: TEXT, name: TEXT, email: TEXT, location: TEXT, username: TEXT, password: TEXT, bio: TEXT, availability: INTEGER, pfp_file_id: INTEGER)

U(id, pn, n, e, l, u, p, b, a)

id -> id, pn, n, e, l, u, p, b, a
pn -> id, pn, n, e, l, u, p, b, a
e  -> id, pn, n, e, l, u, p, b, a
u  -> id, pn, n, e, l, u, p, b, a

id, pn, e, and u are all minimum candidate keys and are the only FDs on the relation so this relation is in BCNF

---

Contacts(**<ins>user_id: INTEGER, contact_user_id: INTEGER</ins>**)

C(u, c)

u, c -> u, c

the only members of this relation are part of the minimum candidate key, this relation is in BCNF

---

Messages(**<ins>message_id:INTEGER</ins>**, message: TEXT, reply_to: INTEGER, attachment: INTEGER, timestamp: BIGINT, last_edited: BIGINT, sender_id: INTEGER, chat_id INTEGER, views: INTEGER, pinned: boolean)

M(id, m, r, a, t, l, s, c, v, b)

id -> id, m, r, a, t, l, s, c, v, b

the only FD within this relation is id which is the min key therefore it is in BCNF

---

Chats (**<ins>chat_id: INTEGER</ins>**, primary_owner: INTEGER, secondary_owner: INTEGER, sending_privilage: INTEGER, track_views: BOOLEAN, max_members INTEGER, chat_name text)

CH(id, p, sec, sen, t, m, c)

id -> id, p, sec, sen, t, m, c

the only FD within this relation is id which is the min key therefore it is in BCNF

---

Chat_Members (**<ins>chat_id: INTEGER, member_id: INTEGER</ins>**, privilage: INTEGER, wants_notifications: BOOLEAN, last_seen INTEGER,)

CM(c, m, p, w, l)

c,m -> c, m, p, w, l

since c,m is the minimum candidate key and the only FD that holds this relation is in BCNF

## q4

```sql
CREATE TABLE files (
    file_id SERIAL PRIMARY KEY,
    file_name VARCHAR(150) NOT NULL,
    contents BYTEA NOT NULL
);

CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    phone_number VARCHAR(15) NOT NULL CHECK(length(name) >= 3) UNIQUE,
    name VARCHAR(150) NOT NULL CHECK(length(name) >= 3),
    email VARCHAR(150) NOT NULL CHECK(length(name) >= 3) UNIQUE,
    location VARCHAR(150) NOT NULL CHECK(length(name) >= 3),
    username VARCHAR(50) NULL CHECK(length(name) >= 3) UNIQUE,
    password VARCHAR(50) NOT NULL CHECK(length(name) >= 3),
    bio VARCHAR(250) NOT NULL DEFAULT '',
    availability INTEGER NOT NULL DEFAULT 0,
    pfp_file_id INTEGER REFERENCES files(file_id)
);

CREATE TABLE contacts (
    user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    contact_user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    PRIMARY KEY(user_id, contact_user_id)
);

CREATE TABLE chats (
    chat_id SERIAL PRIMARY KEY,
    primary_owner INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    secondary_owner INTEGER NULL REFERENCES users(user_id) ON DELETE CASCADE,
    
    sending_privilage INTEGER NOT NULL,

    track_views BOOLEAN NOT NULL,
    max_members INTEGER NOT NULL,
    chat_name VARCHAR(100) NULL
);

CREATE TABLE messages (
    message_id SERIAL PRIMARY KEY,
    message TEXT,
    reply_to INTEGER NULL REFERENCES messages(message_id) ON DELETE SET NULL,
    attachment_id INTEGER NULL REFERENCES files(file_id),
    posted BIGINT NOT NULL,
    last_edited BIGINT,
    sender_id INTEGER NULL REFERENCES users(user_id) ON DELETE SET NULL,
    chat_id INTEGER NOT NULL REFERENCES chats(chat_id) ON DELETE CASCADE,
    views INTEGER NULL,
    pinned BOOLEAN NOT NULL DEFAULT FALSE
);


CREATE TABLE chat_members (
    chat_id INTEGER NOT NULL REFERENCES chats(chat_id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    privilage INTEGER NOT NULL,

    wants_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen BIGINT NOT NULL DEFAULT 0,

    PRIMARY KEY (chat_id, member_id)
);
```

```sql

```

## q5


**a. Retrieve the list of all users**
```sql
SELECT * FROM users
```

**b. Retrieve the list of all online users.**
```sql
SELECT * FROM users WHERE availability=1
```

**c. Given a user (by phone number or unique ID or username), retrieve all information of the user.**
```sql
SELECT
  *
FROM
  users
WHERE
  COALESCE(user_id=:user_id, false)
  OR COALESCE(phone_number=:phone_number, false)
  OR COALESCE(username=:username, false)
```

**d. Given a user (by phone number, unique ID or username) retrieve all his/her chats (private chats, normal groups and channels)**
```sql
SELECT
  chat_id
FROM
  chat_members
WHERE member_id=(
  SELECT
    user_id
  FROM
    users
  WHERE
    COALESCE(user_id=:user_id, false)
    OR COALESCE(phone_number=:phone_number, false)
    OR COALESCE(username=:username, false)
)
```

**e. For a given chat, retrieve its metadata (chat title, bio, join link (if applicable), etc.)**
```sql
SELECT * FROM chats WHERE chat_id=:chat_id
```

**f. For a given chat, retrieve all its users.**
```sql
SELECT member_id FROM chat_members WHERE chat_id=:chat_id
```

**g. For a given chat, retrieve all its online users.**
```sql
SELECT
  member_id
FROM
  chat_members
WHERE
  chat_id=:chat_id
  AND 1=(SELECT availability FROM users WHERE user_id=chat_members.member_id)
```

**h. For a given chat, retrieve its creator.**
```sql
SELECT
  primary_owner
FROM
  chats
WHERE
  chat_id=:chat_id
```

**i. For a given chat, retrieve all its admins (including the creator).**
```sql
SELECT
  member_id
FROM
  chat_members
WHERE
  chat_id=:chat_id
  AND privilage>=(SELECT sending_privilage FROM chats WHERE chat_id=:chat_id)
```

j. For a given chat admin, retrieve his/her permissions.
```sql
SELECT
  privilage
FROM
  chat_members
WHERE
  chat_id=:chat_id
  AND member_id=:member_id
```

k. For a given chat, retrieve all its message history
```sql
SELECT
  message_id
FROM
  messages
WHERE
  chat_id=:chat_id
ORDER BY posted DESC
```

l. For a given chat, retrieve its message during a specific date-time
```sql
SELECT 
  message_id
FROM
  messages
WHERE
  chat_id=:chat_id
  AND posted<=:before
  AND posted>=:after
ORDER BY posted DESC
```

m. For a given chat, retrieve all messages posted by a user during a
```sql
SELECT 
  message_id
FROM
  messages
WHERE
  chat_id=:chat_id
  AND sender_id=:user_id
  AND posted<=:before
  AND posted>=:after
ORDER BY posted DESC
```

n. For a given chat, retrieve its unread messages.
```sql
SELECT
  message_id
FROM
  messages
WHERE
  chat_id=:chat_id
  AND posted>(
    SELECT
      MIN(last_seen)
    FROM
      chat_members
    WHERE
      chat_id=:chat_id
  )
```

o. For a given chat, retrieve the last n (say 100) message.
```sql
SELECT
  message_id
FROM
  messages
WHERE
  chat_id=:chat_id
ORDER BY posted DESC
LIMIT 100
```

p. For a given message ID, retrieve all its information.
```sql
SELECT * FROM messages WHERE message_id=:message_id
```
