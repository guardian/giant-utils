#!/bin/bash
set -e

echo "WARNING: this will take about half an hour and overwrite contents of target dir!"
echo "Cannot be killed with Ctrl+C: go into Docker Desktop Dashboard and delete container to stop"

# Warning: cannot be killed with Ctrl+C! Maybe fix with below:
# https://forums.docker.com/t/docker-run-cannot-be-killed-with-ctrl-c/13108/2
docker run \
  --rm \
  --user "$(id -u)":"$(id -g)" \
  -v "$PWD":/usr/src/giant-utils \
  -w /usr/src/giant-utils \
  amd64/rust:1.64 \
  cargo build --release

echo "Linux x86 executable is now at target/release/giant-utils"

