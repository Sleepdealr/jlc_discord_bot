#!/bin/bash

# Pulls from git
# Comment out to disable
git pull

# Sources cargo env
. "/home/[username]/.cargo/env"

# Format project
cargo fmt

# Runs cargo directly
/home/[username]/.cargo/bin/cargo run --release
