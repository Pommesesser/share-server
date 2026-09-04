#!/bin/bash
set -e

cd /root/share-server

echo "Pulling latest code..."
git pull

echo "Building release..."
cargo build --release

echo "Restarting server..."
systemctl restart share-server

echo "Deployment complete."
systemctl --no-pager status share-server