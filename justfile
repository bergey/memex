update:
   cargo +nightly update -Z unstable-options --breaking

sqlx-cli:
    cargo install sqlx-cli --no-default-features --features rustls,postgres
