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

rm -rf cache_fs vault_fs

# docker exec -t maestro-mongo-1 mongosh -u $MONGO_USERNAME -p $MONGO_PASSWORD --eval "use logs" \
    # --eval 'db.diskWakeUp.insertOne({{  "_id": {    ObjectId"61e2b2cc16fbdf1b5e3e5c60")  },  "diskId": {    ObjectId"14361e456568c1aef56e765a")  },  "startup": {    "date": {      "$date": {        "$numberLong": "1642032000000"      }    },    "isManual": false  },  "shutdown": null,  "periodInfo": null}})' ## Insert diskwakeup for test purposes

DISK_WAKE_UP='db.diskWakeup.insertOne(
{
  "diskId": ObjectId("655ceb05ee2884fd5e16872b"),
  "startup": {
    "date": {
      "$date": {
        "$numberLong": "1642032000000"
      }
    },
    "isManual": false
  },
  "shutdown": null,
  "periodInfo": null
})'

docker exec -t maestro-mongo-1 mongosh -u "$MONGO_USERNAME" -p "$MONGO_PASSWORD" --eval "use logs" --eval "$DISK_WAKE_UP"

cargo test -- --nocapture --test-threads=1 --color always

tree cache_fs vault_fs
