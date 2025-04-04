#!/bin/bash
# Verify the database schema

echo "Verifying tables..."
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\dt"

echo -e "\nUsers table structure:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\d users"

echo -e "\nFolders table structure:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\d folders"

echo -e "\nFiles table structure:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\d files"

echo -e "\nVerifying indexes on users:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\di idx_users*"

echo -e "\nVerifying indexes on folders:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\di idx_folders*"

echo -e "\nVerifying indexes on files:"
docker-compose -f docker-compose.test.yml exec -T db_test psql -U postgres -d marble_test -c "\di idx_files*"
