#!/bin/bash

# Create release directory
mkdir -p release

# Remove existing release files
rm -rf release/*

# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --bin tomatwo

# Build for Linux
# cargo build --release --target x86_64-unknown-linux-gnu
# cargo build --release --target x86_64-unknown-linux-gnu --bin tomatwo

# Build for macOS
# cargo build --release --target x86_64-apple-darwin
# cargo build --release --target x86_64-apple-darwin --bin tomatwo

# Build library
cargo build --release --lib

# Create directories for each platform
mkdir -p release/windows
# mkdir -p release/linux
# mkdir -p release/macos
mkdir -p release/lib

# Copy Windows binaries
cp target/x86_64-pc-windows-msvc/release/gooey_tomatwo.exe release/windows/
cp target/x86_64-pc-windows-msvc/release/tomatwo.exe release/windows/

# Copy Linux binaries
# cp target/x86_64-unknown-linux-gnu/release/gooey_tomatwo release/linux/
# cp target/x86_64-unknown-linux-gnu/release/tomatwo release/linux/

# Copy macOS binaries
# cp target/x86_64-apple-darwin/release/gooey_tomatwo release/macos/
# cp target/x86_64-apple-darwin/release/tomatwo release/macos/

# Copy library
cp target/release/libtomatwo_seed.rlib release/lib/

echo "Build complete. Release files are in the 'release' directory."