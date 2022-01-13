#!/bin/bash

# Pulls from git
# Comment out to disable
git pull

# Sources cargo env
. "~/.cargo/env"

# Format project
cargo fmt

# Runs cargo directly
~/.cargo/bin/cargo run --release
