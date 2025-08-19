create table users(
    id integer not null primary key autoincrement,
    email varchar not null,
    password varchar not null,
    updated_at datetime not null default current_timestamp,
    created_at datetime not null default current_timestamp
);
create unique index users_table_email_index on users (email);

create table sessions(
    id integer not null primary key autoincrement,
    user_id integer not null,
    uuid varchar not null,
    csrf_token varchar not null,
    issued_at datetime not null,
    device_info text,
    ip_address varchar,
    created_at datetime not null default current_timestamp,
    updated_at datetime not null default current_timestamp
);
create unique index sessions_table_uuid_index on sessions (uuid);
create index sessions_table_user_id_index on sessions (user_id);
