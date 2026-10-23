update:
   # requires cargo-edit
   cargo upgrade

[working-directory: 'server']
sqlx:
    #!/usr/bin/env bash
    set -euxo pipefail
    cargo sqlx prepare
    git reset
    git add .sqlx
    if ! git diff --cached --exit-code  --quiet; \
        then git commit -m "sqlx prepare"; \
        else echo "nothing to commit"; \
        fi

sqlx-cli:
    cargo install sqlx-cli --no-default-features --features rustls,postgres

pg_db := "memex-dev"

setup:
 createuser memex --login -P || true
 createdb {{pg_db}} -O memex || true
 psql -d {{pg_db}} -c 'grant all on database "{{pg_db}}" to memex' -c 'grant all on all tables in schema public to memex' -c 'grant all on schema public to memex'

release-server:
    cargo build --release --bin memex-server

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

# .env file instead?
[env("DATABASE_URL", "postgres://memex:memex@localhost/memex")]
test:
  # all packages with test suites, not only the native executables in default-members
  cargo test -p memex-shared -p memex-query -p memex-cli -p memex-server
