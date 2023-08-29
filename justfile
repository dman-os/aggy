set shell := ["sh", "-c"]
set dotenv-load

# List the avail commands
default:
  @just --list --unsorted

pre-commit:
  cargo fmt
  @just gen-sqlx-offline

CARGO_TARGET_DIR := "/var/run/media/asdf/Windows/target/aggy/" 
SQLX_TMP := "/var/run/media/asdf/Windows/target/aggy/sqlx-tmp" 
SQLX_OFFLINE_DIR := "/var/run/media/asdf/Windows/target/aggy/sqlx-final"
gen-sqlx-offline:
  mkdir -p .sqlx && mkdir -p {{SQLX_TMP}} && mkdir -p {{SQLX_OFFLINE_DIR}}
  rm .sqlx/query-*.json  || true
  # force full recomplile of crates that use sqlx queries
  cargo clean -p aggy_api
  cargo clean -p epigram_api
  cargo clean -p qtrunk_api
  SQLX_TMP={{SQLX_TMP}} \
  SQLX_OFFLINE_DIR={{SQLX_OFFLINE_DIR}} \
  cargo check
  cp /var/run/media/asdf/Windows/target/aggy/sqlx-final/* .sqlx -r

run-anf:
  cargo run -p aggynfrens_api
