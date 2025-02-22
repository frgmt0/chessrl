#!/bin/bash

# Ensure Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Clone the repository
git clone https://github.com/frgmt0/chessrl.git
cd chessrl

# Build and install
cargo build --release
sudo mv target/release/chess /usr/local/bin/chessrl

echo "ChessRL has been installed successfully!"
echo "Start a new game by typing: chessrl"
