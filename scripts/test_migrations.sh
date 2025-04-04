#!/bin/bash
# Test migrations on the PostgreSQL 17 test database

set -e

# Check if Docker Compose is running, if not start it
if ! docker-compose -f docker-compose.test.yml ps | grep -q "Up"; then
  echo "Starting PostgreSQL 17 test database..."
  docker-compose -f docker-compose.test.yml up -d
fi

# Wait for it to be ready
echo "Waiting for test database to be ready..."
until docker-compose -f docker-compose.test.yml exec -T db_test pg_isready -U postgres; do
  sleep 1
done

# Create .env.test file
cat > .env.test << EOF
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/marble_test
EOF

echo "Test database is ready at postgres://postgres:postgres@localhost:5433/marble_test"

# Reset the database for a clean test
echo "Resetting test database..."
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

# Run the migrations using SQLx CLI
if command -v sqlx &> /dev/null; then
  echo "Running migrations with SQLx CLI..."
  SQLX_OFFLINE=false DATABASE_URL=postgres://postgres:postgres@localhost:5433/marble_test sqlx migrate run --source crates/marble-db/migrations
else
  echo "SQLx CLI not found, attempting to install..."
  ~/.cargo/bin/cargo install sqlx-cli --no-default-features --features postgres
  echo "Running migrations with SQLx CLI..."
  SQLX_OFFLINE=false DATABASE_URL=postgres://postgres:postgres@localhost:5433/marble_test ~/.cargo/bin/sqlx migrate run --source crates/marble-db/migrations
fi

echo "Migrations completed successfully!"

# Verify the tables exist
echo "Verifying tables..."
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\dt"

# Verify table structure
echo -e "\nUsers table structure:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\d users"

echo -e "\nFolders table structure:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\d folders"

echo -e "\nFiles table structure:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\d files"

echo -e "\nMigration test completed successfully!"
