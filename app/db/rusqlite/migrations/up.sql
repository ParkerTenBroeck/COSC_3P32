CREATE TABLE files (
    file_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    file_name VARCHAR(150) NOT NULL,
    contents BLOB NOT NULL
);

CREATE TABLE users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    phone_number VARCHAR(15) NOT NULL CHECK(length(name) > 5),
    name VARCHAR(150) NOT NULL CHECK(length(name) > 3),
    email VARCHAR(150) NOT NULL CHECK(length(name) > 5),
    location VARCHAR(150) NOT NULL CHECK(length(name) > 5),
    username VARCHAR(50) NOT NULL CHECK(length(name) > 5),
    password VARCHAR(50) NOT NULL CHECK(length(name) > 5),
    bio VARCHAR(250) NOT NULL DEFAULT "",
    availability TINYINT NOT NULL,
    php_file_id INTEGER REFERENCES files(file_id)
);

CREATE TABLE contacts (
    user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    contact_user_id INTEGER  REFERENCES users(user_id) ON DELETE CASCADE,
    PRIMARY KEY(user_id, contact_user_id)
);

CREATE TABLE messages (
    message_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    message TEXT,
    attachment INTEGER NULL REFERENCES files(file_id),
    posted BIGINT NOT NULL,
    last_edited BIGINT,

    private_message_id INTEGER NULL REFERENCES private_messages(message_id) ON DELETE CASCADE,
    channel_message_id INTEGER NULL REFERENCES private_messages(message_id) ON DELETE CASCADE,
    group_message_id INTEGER NULL REFERENCES private_messages(message_id) ON DELETE CASCADE,
    
    CONSTRAINT SingleKind 
        CHECK (
            (private_message_id IS NOT NULL) 
            + (channel_message_id IS NOT NULL) 
            + (group_message_id IS NOT NULL) <= 1
            )
);

CREATE TABLE private_messages (
    from_id INTEGER REFERENCES users(user_id) ON DELETE CASCADE,
    to_id INTEGER REFERENCES users(user_id) ON DELETE CASCADE,
    message_id INTEGER REFERENCES messages(message_id) ON DELETE CASCADE,
    PRIMARY KEY(message_id)
);

CREATE TABLE channels (
    channel_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    channel_name VARCHAR(150) NOT NULL,
    owner_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE TABLE channel_memebers (
    channel_id INTEGER NOT NULL REFERENCES channels(channel_id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    privilage TINYINT NOT NULL,
    PRIMARY KEY(channel_id, user_id)
);

create TABLE channel_messages (
    from_id INTEGER REFERENCES users(user_id) ON DELETE SET NULL,
    message_id INTEGER REFERENCES messages(message_id) ON DELETE CASCADE,
    channel_id INTEGER REFERENCES channels(channel_id) ON DELETE CASCADE,
    views INTEGER NOT NULL,
    PRIMARY KEY(message_id)
);


CREATE TABLE groups (
    group_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    group_name VARCHAR(150) NOT NULL,
    owner_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE TABLE group_memebers (
    group_id INTEGER NOT NULL REFERENCES groups(group_id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    PRIMARY KEY(group_id, user_id)
);

create TABLE group_messages (
    from_id INTEGER REFERENCES users(user_id) ON DELETE SET NULL,
    message_id INTEGER REFERENCES messages(message_id) ON DELETE CASCADE,
    group_id INTEGER REFERENCES groups(group_id) ON DELETE CASCADE,
    PRIMARY KEY(message_id)
);