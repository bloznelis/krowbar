# This menu
help:
  just --list

# Build the binary
build:
  cargo build --release

# Release a new version
release:
  ./scripts/release.sh

install:
  cargo install --path .
