#!/bin/bash

set -o allexport
source ./env/vault.env
set +o allexport

cargo build
cargo run
