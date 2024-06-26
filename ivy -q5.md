#5

**a. Retrieve the list of all users**

```sql
SELECT * FROM users;
```

**b. Retrieve the list of all online users.**

availability is one when the user is online, we have availability as an integer instead of a boolean to allow for a user to set it to do not disturb or invisible (always appearing offline)

```sql
SELECT * FROM users WHERE availability = 1;
```

**c. Given a user (by phone number or unique ID or username), retrieve all information of the user.**

```sql
SELECT * FROM users WHERE user_id = ___ OR phone_number = ___ OR username = ___;
```

**d. Given a user (by phone number, unique ID or username) retrieve all his/her chats (private chats, normal groups and channels)**

```sql
SELECT chats.* FROM chats INNER JOIN chat_members ON chats.chat_id=chat_members.chat_id AND chat_members.member_id = __;
```

**e. For a given chat, retrieve its metadata (chat title, bio, join link (if applicable), etc.)**

for our production database we are just using the id of a chat as its join link

```sql
SELECT (chat_id, primary_owner, secondary_owner, sending_privilage, track_views, max_members, chat_name) FROM chats where chat_id = __;
```

**f. For a given chat, retrieve all its users.**

```sql
SELECT users.* FROM chat_members INNER JOIN users ON chat_members.member_id = users.user_id WHERE chat_id = __;
```

**g. For a given chat, retrieve all its online users.**

```sql
SELECT users.* FROM chat_members INNER JOIN users ON chat_members.member_id = users.user_id WHERE chat_members.chat_id = __ AND users.availability = 1;
```

**h. For a given chat, retrieve its creator.**

```sql
SELECT * FROM users WHERE user_id =(SELECT primary_owner FROM chats where chat_id = __);
```

**i. For a given chat, retrieve all its admins (including the creator).**

```sql
SELECT users.* FROM chat_members INNER JOIN users ON chat_members.member_id = users.user_id WHERE chat_members.chat_id = __ AND chat_members.privilage = 1;
```

j. For a given chat admin, retrieve his/her permissions.

***parker we gotta talk about this one***

k. For a given chat, retrieve all its message history



l. For a given chat, retrieve its message during a specific date-time
range.



m. For a given chat, retrieve all messages posted by a user during a
specific date-time range.



n. For a given chat, retrieve its unread messages.



o. For a given chat, retrieve the last n (say 100) message.



p. For a given message ID, retrieve all its information.



Important note concerning queries:
---
***The queries must be implemented either in PostgreSQL, MySQL, MS SQL Server, or Oracle.***
