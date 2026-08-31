create table passkeys (
  cred_id bytea primary key,
  user_id integer not null references users(id),
  value bytea not null
  );

create table passkey_challenges (
  challenge bytea not null primary key,  -- 32 random bytes from webauthn-rs
  user_id integer not null references users(id),
  state bytea not null,
  expires timestamptz not null default now() + interval '5min'
  );
