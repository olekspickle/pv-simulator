#!/bin/bash

set -e

# start RabbitMQ container
docker compose up


curl -O https://github.com/olekspickle/pv-simulator/releases/download/0.0.1/pv-simulator
RUST_LOG=info

# run program
./pv-simulator
