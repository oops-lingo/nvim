# pipefail is required: the test step pipes through grep, and without it the exit
# code comes from the last command in the pipe, so a failing test run slips through.
set shell := ["bash", "-euo", "pipefail", "-c"]

# Minimum Checklist
check:
  cargo check --quiet
  cargo fmt --check
  cargo clippy --quiet
  cargo test --quiet

# List of checks required before publication
precommit:
  RUSTFLAGS="-D warnings" cargo check 
  cargo fmt --check
  cargo clippy -- -D warnings
  cargo test
  cargo audit
  cargo build
  cargo build --release



# Launch nvim with a local plugin loaded. This includes, among other things, the user configuration.
nvim path = "src/main.rs": 
    nvim -c 'lua vim.opt.rtp:prepend(vim.fn.getcwd()); require("test_run")' {{path}}


# Launch nvim with a local plugin loaded. With the `--clean` flag. Default configuration 
nvimc path = "src/main.rs": 
    nvim --clean -c 'lua vim.opt.rtp:prepend(vim.fn.getcwd()); require("test_run")' {{path}}
