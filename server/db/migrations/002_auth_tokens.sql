create table auth_tokens (
  id uuid primary key,
  user_id integer references users(id),
  expires timestamptz
 );
