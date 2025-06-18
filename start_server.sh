#!/bin/bash

# TAMS Rust Server Startup Script
# This script sets the required environment variables and starts the server

set -e

echo "🚀 Starting TAMS Rust server..."

# Ensure directories exist
mkdir -p data
mkdir -p media_storage
mkdir -p temp_uploads

# Get absolute path to database
DB_PATH="$(pwd)/data/tams.db"

# Set DATABASE_URL for SQLx runtime with absolute path
export DATABASE_URL="sqlite:${DB_PATH}"

# Optional: Set log level for more verbose output
# export RUST_LOG=debug

echo "📊 Environment:"
echo "  DATABASE_URL: $DATABASE_URL"
echo "  RUST_LOG: ${RUST_LOG:-info}"
echo "  Database file: $DB_PATH"

# Check if database file exists and is readable
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Database file does not exist: $DB_PATH"
    echo "🔧 Creating database..."
    sqlite3 "$DB_PATH" < create_db.sql
    echo "✅ Database created"
elif [ ! -r "$DB_PATH" ]; then
    echo "❌ Database file is not readable: $DB_PATH"
    echo "🔧 Fixing permissions..."
    chmod 644 "$DB_PATH"
    echo "✅ Permissions fixed"
else
    echo "✅ Database file exists and is readable"
fi

# Test database connection
echo "🔍 Testing database connection..."
if sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master;" > /dev/null 2>&1; then
    echo "✅ Database connection successful"
else
    echo "❌ Database connection failed, recreating database..."
    rm -f "$DB_PATH"
    sqlite3 "$DB_PATH" < create_db.sql
    echo "✅ Database recreated"
fi

echo ""
echo "🔧 Building and starting server..."

# Build and run the server
cargo run

echo ""
echo "✅ Server stopped" 