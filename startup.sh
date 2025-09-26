#!/bin/bash
set -e

echo "Setting up database..."
sqlx database create

echo "Starting mrkt..."
exec ./mrkt