#!/usr/bin/env bash

set -ex

cargo build -r
RUST_LOG=info solana-test-validator --geyser-plugin-config ./config.json