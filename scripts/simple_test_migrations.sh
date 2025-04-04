#!/bin/bash
# Simple test to run migrations using Rust code

set -e

# Ensure the test database is running
if ! docker ps | grep -q "marble-next-db_test"; then
  echo "Starting PostgreSQL test database..."
  docker-compose -f docker-compose.test.yml up -d
  
  # Wait for it to be ready
  echo "Waiting for database to be ready..."
  sleep 5
fi

# Run the migrations using our Rust test
cd ~/code/marble-next
export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5433/marble_test"
~/.cargo/bin/cargo test -p marble-db migration_tests
