begin;

create table profile (
  id serial primary key,
  username text unique not null,
  password text not null
);

create table token (
  id serial primary key,
  token text unique not null,
  issued_at timestamptz not null,
  profile_id int references profile(id) not null
);

create table permission (
  id serial primary key
);

create table permission_token (
  id serial primary key,
  permission_id int references permission(id) not null,
  token_id int references token(id) not null
);

commit;