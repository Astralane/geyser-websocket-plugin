#!/bin/bash

set -ex

cargo build
solana-test-validator --geyser-plugin-config config.json