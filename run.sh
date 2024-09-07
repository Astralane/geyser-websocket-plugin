#!/bin/bash

set -ex

cargo build -r
RUST_LOG=info && solana-test-validator --log --geyser-plugin-config config.json