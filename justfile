set positional-arguments

# List recipes
default:
  @just --list

# Build the site into dist/. CI passes the public address as base_url.
build base_url="http://localhost:8000":
  rm -rf dist
  cargo run --release -- build --out dist --base-url {{base_url}}

# Validate site.toml and events/ without writing anything
check:
  cargo run --release -- check

# Print true if an event ended in the last 48 hours, so a scheduled rebuild has work to do
due:
  @cargo run --release --quiet -- due

# Run the tests
test:
  cargo test

# Format check, clippy and spelling
lint:
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  typos

# Dependency advisories, licenses and sources
audit:
  cargo deny check

# Serve dist/ at http://localhost:8000
serve:
  python3 -m http.server --directory dist 8000
