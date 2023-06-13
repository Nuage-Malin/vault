#!/bin/bash

rm -rf 64781e773236c1aef30e6189
set -o allexport
source ./env/vault.env
set +o allexport


cargo test -- --nocapture --test-threads=1
