#!/bin/bash

set -o allexport
source ./env/mongo.env
source ./env/vault.env
set +o allexport

cargo update
cargo build && cargo run
