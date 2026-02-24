#!/bin/bash
# Wrapper for systemd-nspawn to work without systemd-machined
# Adds --register=no --keep-unit flags to prevent registration attempts
# which fail when running in a container without systemd as PID 1
exec /usr/bin/systemd-nspawn --register=no --keep-unit "$@"
