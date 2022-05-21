-- Your SQL goes here

create table passwords (
  id integer not null primary key,
  password text not null unique,
  notes text not null default "",
  created_at text not null
);

create unique index password_idx
  on passwords (password);
