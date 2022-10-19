#!/bin/bash

# Warning: cannot be killed with Ctrl+C! Maybe fix with below:
# https://forums.docker.com/t/docker-run-cannot-be-killed-with-ctrl-c/13108/2
docker run \
  --rm \
  --user "$(id -u)":"$(id -g)" \
  -v "$PWD":/usr/src/giant-utils \
  -w /usr/src/giant-utils \
  rust:1.64 \
  cargo build --release

