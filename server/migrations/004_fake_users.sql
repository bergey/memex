CREATE OR REPLACE FUNCTION create_fake_users(n_users integer) RETURNS integer AS $$
declare
i integer := 1;
user_id integer;
user_eid uuid;
token uuid;
begin
loop
user_eid = format('%s-0000-0000-0000-000000000000', lpad(to_hex(i), 8, '0'))::uuid;
token = format('00000000-0000-0000-0000-%s', lpad(to_hex(i), 12, '0'))::uuid;
insert into users (external_id) values (user_eid) returning id into user_id;
  insert into auth_tokens (user_id, id, expires) values (user_id, token, now() + interval '24 hours');
  i = i + 1;
  exit when i > n_users;
end loop;
return i;
end;
$$ LANGUAGE plpgsql;
