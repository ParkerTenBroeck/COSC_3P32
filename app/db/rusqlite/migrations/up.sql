CREATE TABLE files (
    file_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    file_name VARCHAR(150) NOT NULL,
    contents BLOB NOT NULL
);

CREATE TABLE users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    phone_number VARCHAR(15) NOT NULL CHECK(length(name) >= 3) UNIQUE,
    name VARCHAR(150) NOT NULL CHECK(length(name) >= 3),
    email VARCHAR(150) NOT NULL CHECK(length(name) >= 3) UNIQUE,
    location VARCHAR(150) NOT NULL CHECK(length(name) >= 3),
    username VARCHAR(50) NULL CHECK(length(name) >= 3) UNIQUE,
    password VARCHAR(50) NOT NULL CHECK(length(name) >= 3),
    bio VARCHAR(250) NOT NULL DEFAULT "",
    availability TINYINT NOT NULL DEFAULT 0,
    pfp_file_id INTEGER REFERENCES files(file_id)
);

CREATE TABLE contacts (
    user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    contact_user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    PRIMARY KEY(user_id, contact_user_id)
);

CREATE TABLE messages (
    message_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
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

CREATE TABLE chats (
    chat_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    primary_owner INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    secondary_owner INTEGER NULL REFERENCES users(user_id) ON DELETE CASCADE,
    
    sending_privilage UNSIGNED TINYINT NOT NULL,

    track_views BOOLEAN NOT NULL,
    max_members UNSIGNED INTEGER NOT NULL,
    chat_name VARCHAR(100) NULL
);

CREATE TABLE chat_members (
    chat_id INTEGER NOT NULL REFERENCES chats(chat_id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    privilage UNSIGNED TINYINT NOT NULL,

    wants_notifications BOOLEAN NOT NULL DEFAULT TRUE,
    last_seen BIGINT NOT NULL DEFAULT 0,

    PRIMARY KEY (chat_id, member_id)
);