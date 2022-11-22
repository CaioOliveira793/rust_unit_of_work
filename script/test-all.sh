#!/usr/bin/env bash

set -o errexit;

export $(grep -v '^#' .env | xargs);

cargo test --tests;

cargo run --example pg_deadpool --features=pg_deadpool;

cargo run --example sqlx --features=sqlx;
