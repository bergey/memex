update:
   cargo +nightly update -Z unstable-options --breaking

sqlx-cli:
    cargo install sqlx-cli --no-default-features --features rustls,postgres

setup:
 createuser memex --login || true
 createdb {{pg_db}} -O memex || true
 psql -d {{pg_db}} -c 'grant all on database "{{pg_db}}" to memex' -c 'grant all on all tables in schema public to memex' -c 'grant all on schema public to memex'

release-server:
    cargo build --release --bin server

release-client:
    just client/release

[parallel]
deploy: release-client release-server
    sudo cp target/release/server /usr/bin/memex-server --backup=numbered
    sudo systemctl restart memex
    # server should never be older than client
    sudo cp -r client/dist/. /usr/share/caddy/memex --backup=numbered

# after verifying deploy worked
deploy-clean:
    rm /usr/share/caddy/memex/*~
    rm /usr/bin/memex-server*~

pg_db := "memex-dev"
