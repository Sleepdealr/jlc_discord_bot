#!/bin/bash

# Pulls from git
# Sometimes it doesn't feel like pulling so you need to stash and reset it
# There is probably a better way of doing this
# Comment out to disable
git stash
git stash drop
git pull

# Sources cargo env
. "~/.cargo/env"

# Format project
cargo fmt

# Runs cargo directly
~/.cargo/bin/cargo run --release
