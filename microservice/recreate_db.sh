#!/bin/bash

DB_URL="sqlite://data.db"

rm -f data.db
sqlx database create --database-url="$DB_URL"
sqlx migrate run --database-url="$DB_URL" --source database/migrations
