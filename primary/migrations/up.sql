create table profile (
  id serial primary key,
  username text unique not null,
  password text not null
);