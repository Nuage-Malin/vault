#!/bin/bash

rm -rf 64781e773236c1aef30e6189
cargo test -- --nocapture --test-threads=1
