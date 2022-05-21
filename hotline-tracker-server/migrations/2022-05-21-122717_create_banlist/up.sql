-- Your SQL goes here

create table banlist (
  id integer not null primary key,
  address text not null default "" unique,
  notes text not null default "",
  created_at text not null
);

create unique index banlist_address_idx
  on banlist ( address );
