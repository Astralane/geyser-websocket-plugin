#!/bin/bash

set -ex

cargo build -r
RUST_LOG=geyser_test_plugin::plugin=info && solana-test-validator --log --geyser-plugin-config config.json