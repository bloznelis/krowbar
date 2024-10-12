# This menu
help:
  just --list

# Perform check
check:
  cargo check

# Build the binary
build:
  cargo build --release

# Release a new version
release:
  ./scripts/release.sh

install:
  cargo install --path .
