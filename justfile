# pipefail is required: the test step pipes through grep, and without it the exit
# code comes from the last command in the pipe, so a failing test run slips through.
set shell := ["bash", "-euo", "pipefail", "-c"]

check:
  cargo check --quiet
  cargo fmt --check
  cargo clippy --quiet
  cargo test --quiet

precommit:
  RUSTFLAGS="-D warnings" cargo check 
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test
  cargo audit
  cargo build
  cargo build --release

