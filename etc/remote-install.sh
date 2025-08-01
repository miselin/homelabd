#!/bin/bash

set -euo pipefail

INSTALLER_USER=$1

systemctl stop homelabd || true

BASE_DIR=/home/${INSTALLER_USER}
if [ "${INSTALLER_USER}" == "root" ]; then
    BASE_DIR="/root"
fi

cp ${BASE_DIR}/homelabd /usr/local/bin/homelabd
cp ${BASE_DIR}/homelabd.service /etc/systemd/system/homelabd.service

chmod +x /usr/local/bin/homelabd

systemctl daemon-reload
systemctl enable homelabd
systemctl start homelabd
systemctl status homelabd
