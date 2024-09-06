#!/bin/bash

set -ex

cargo build -r
solana-test-validator --geyser-plugin-config config.json