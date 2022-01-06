#!/bin/bash

# Pulls from git
# Comment out to disable
git pull

# Format project
cargo fmt

# Sources cargo env
. "/home/[username]/.cargo/env"

# Runs cargo directly
/home/[username]/.cargo/bin/cargo run --release
