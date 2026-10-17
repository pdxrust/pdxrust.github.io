set positional-arguments

# List recipes
default:
  @just --list

# Build the site into dist/
build:
  rm -rf dist
  cargo run --release -- build --out dist

# Validate site.toml and events/ without writing anything
check:
  cargo run --release -- check

# Run the tests
test:
  cargo test

# Format check and clippy
lint:
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings

# Dependency advisories, licenses and sources
audit:
  cargo deny check

# Serve dist/ at http://localhost:8000
serve:
  python3 -m http.server --directory dist 8000
