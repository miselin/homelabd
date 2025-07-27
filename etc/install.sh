#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$(dirname $(readlink -f "$0"))

HOST=$1
if [ -z "$HOST" ]; then
    echo "Usage: $0 <host>"
    exit 1
fi

scp ${SCRIPT_DIR}/../target/x86_64-unknown-linux-gnu/release/homelabd $HOST:homelabd
scp ${SCRIPT_DIR}/../etc/homelabd.service $HOST:homelabd.service
scp ${SCRIPT_DIR}/../etc/remote-install.sh $HOST:remote-install.sh

ssh -t $HOST 'chmod +x remote-install.sh && sudo ./remote-install.sh $USER'
