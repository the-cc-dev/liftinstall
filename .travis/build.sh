#!/usr/bin/env bash
cd /liftinstall

apt update
apt install -y libwebkit2gtk-4.0-dev

curl https://sh.rustup.rs -sSf | sh -s -- -y
export PATH=~/.cargo/bin:$PATH

cargo build
