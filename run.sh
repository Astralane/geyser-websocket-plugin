#!/bin/bash

set -ex

cargo build -r
RUST_LOG=geyser=info && solana-test-validator --log --geyser-plugin-config config.json