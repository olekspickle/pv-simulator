#!/bin/bash

set -e

# start RabbitMQ container
docker compose up

RUST_LOG=info

# run program
cargo run