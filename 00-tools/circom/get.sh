#!/bin/sh

rm -rf circom
git clone https://github.com/iden3/circom.git
cd circom
cargo build --release
cp ./target/release/circom ../bin/
cargo install --path circom
cd ..
rm -rf circom
