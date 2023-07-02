#!/bin/bash

rm -rf 64781e773236c1aef30e6189
set -o allexport
source ./env/vault.env
set +o allexport

usage()
{
    echo "Usage: $0 [--help] [--backtrace]"
    echo -e "\t--help: Prints this message"
    echo -e "\t--backtrace: Function call backtrace is displayed, thanks to env variable RUST_BACKTRACE=1"
    exit 0
}


for arg in "$@"; do
    case $arg in
        --help)
            usage
        ;;
	-h)
            usage
	;;
        --backtrace)
            export RUST_BACKTRACE=1
        ;;
        *)
            echo "Invalid option: $arg" >&2
            exit 1
        ;;
    esac
done

cargo test -- --nocapture --test-threads=1 --color always
