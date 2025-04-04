#!/bin/bash
# Setup test database for marble-db testing

# Start the test database
docker-compose -f docker-compose.test.yml up -d

# Wait for it to be ready
echo "Waiting for test database to be ready..."
until docker-compose -f docker-compose.test.yml exec -T db_test pg_isready -U postgres; do
  sleep 1
done

# Create .env.test file
cat > .env.test << EOF
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5433/marble_test
SQLX_OFFLINE=true
EOF

echo "Test database is ready at postgres://postgres:postgres@localhost:5433/marble_test"
