#!/bin/bash

# TAMS Rust Setup Script
# This script sets up the database and resolves SQLx compilation issues

set -e

echo "🚀 Setting up TAMS Rust server..."

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p data
mkdir -p media_storage
mkdir -p temp_uploads

# Create the database
echo "🗄️ Creating database..."
if [ ! -f data/tams.db ]; then
    sqlite3 data/tams.db < create_db.sql
    echo "✅ Database created successfully"
else
    echo "⚠️  Database already exists, skipping creation"
fi

# Set DATABASE_URL for SQLx
export DATABASE_URL="sqlite:./data/tams.db"

echo "🔧 Setting up SQLx..."
echo "DATABASE_URL=$DATABASE_URL" > .env

# Install sqlx-cli if not present
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features sqlite
fi

# Prepare SQLx queries (this resolves the compilation errors)
echo "⚡ Preparing SQLx queries..."
sqlx database create || true  # Create database if it doesn't exist
sqlx migrate run || echo "No migrations directory, skipping..."

# Try to prepare queries
echo "🔍 Preparing query cache..."
cargo sqlx prepare || echo "⚠️  Query preparation failed - you may need to run 'cargo sqlx prepare' manually after fixing any remaining issues"

echo ""
echo "✅ Setup complete!"
echo ""
echo "🎯 Next steps:"
echo "1. Fix any remaining compilation issues:"
echo "   cargo check"
echo ""
echo "2. Run the server:"
echo "   cargo run"
echo ""
echo "3. Or run in development mode with debug logging:"
echo "   RUST_LOG=debug cargo run"
echo ""
echo "4. The server will be available at: http://localhost:8080"
echo "5. API documentation: http://localhost:8080/"
echo ""
echo "📋 Configuration:"
echo "- Database: data/tams.db"
echo "- Media storage: media_storage/"
echo "- Temp uploads: temp_uploads/"
echo "- Config file: config.toml"
echo ""
echo "🔧 Troubleshooting:"
echo "- If SQLx errors persist, run: cargo sqlx prepare"
echo "- If database issues occur, delete data/tams.db and re-run this script"
echo "- Check logs with: RUST_LOG=debug cargo run" 