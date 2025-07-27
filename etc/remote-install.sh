#!/bin/bash

set -euo pipefail

INSTALLER_USER=$1

systemctl stop homelabd || true

cp /home/${INSTALLER_USER}/homelabd /usr/local/bin/homelabd
cp /home/${INSTALLER_USER}/homelabd.service /etc/systemd/system/homelabd.service

chmod +x /usr/local/bin/homelabd

systemctl daemon-reload
systemctl enable homelabd
systemctl start homelabd
systemctl status homelabd
