#!/bin/bash

ARG_BUILD=false     # --build (vault-cache)
ARG_DOCKER=false    # --docker
ARG_DRY_RUN=false   # --dry-run

set -o allexport
source ./env/mongo.env
source ./env/vault-cache.env
set +o allexport

usage()
{
    echo "Usage: $0 [--help] [--build] [--docker] [--dry-run]"
    echo -e "\t--help: Prints this message"
    echo -e "\t--build: Build vault-cache (cargo build)"
    echo -e "\t--docker: Launches vault-cache with docker"
    echo -e "\t--dry-run: Build vault-cache if --build is specified, but don't launch it"
    exit 0
}

exit_gracefully()
{
    if [ $1 -ne 0 ]; then
        echo -e "\033[31mExiting gracefully...\033[0m" 1>&2
    else
        echo "Exiting gracefully..."
    fi

    if $ARG_DOCKER; then
        docker compose --env-file ./env/vault-cache.env --profile launch down
    else
        docker compose --env-file ./env/local.env --profile mongo down
    fi

    exit $1
}

check_exit_failure()
{
    EXIT_STATUS=$?
    if [ $EXIT_STATUS -ne 0 ]; then
        echo -e "\033[31m$1\033[0m" 1>&2
        exit_gracefully $EXIT_STATUS
    fi
}

for arg in "$@"; do
    case $arg in
        --help)
            usage
        ;;
        --build)
            ARG_BUILD=true
        ;;
        --docker)
            ARG_DOCKER=true
        ;;
        --dry-run)
            ARG_DRY_RUN=true
        ;;
        *)
            echo "Invalid option: $arg" >&2
            exit 1
        ;;
    esac
done

if $ARG_BUILD; then
    if $ARG_DOCKER; then
        docker compose --env-file ./env/mongo.sh --env-file ./env/vault-cache.env build
        check_exit_failure "Failed to build with docker"
    else
        cargo update
        check_exit_failure "Failed to cargo update"

        cargo build
        check_exit_failure "Failed to cargo build vault-cache"
    fi
fi

trap "exit_gracefully 1" SIGINT

if ! $ARG_DRY_RUN; then
    if $ARG_DOCKER; then
        docker compose --env-file ./env/mongo.sh --env-file ./env/vault-cache.env up
        check_exit_failure "Failed to run vault-cache with docker"
    else
        # set -o allexport
        # source ./env/local.env
        # set +o allexport

        cargo run
        check_exit_failure "Failed to run vault-cache locally"
    fi
fi


