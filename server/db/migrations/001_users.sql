create table users (
       id serial primary key,
       external_id uuid not null,
       automerge bytea
);

alter table libraries add column owner integer references users(id);
