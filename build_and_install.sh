#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "Building eww-data-provider in release mode..."
cargo build --release --manifest-path eww-data-provider/Cargo.toml

echo "Building eww-data-requester in release mode..."
cargo build --release --manifest-path eww-data-requester/Cargo.toml

echo "Copying binaries to /usr/bin/ (requires sudo password)..."
sudo cp eww-data-provider/target/release/eww-data-provider /usr/bin/
sudo cp eww-data-requester/target/release/eww-data-requester /usr/bin/

echo "Installation complete."