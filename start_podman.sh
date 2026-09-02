#!/bin/bash
echo "Starting Helios via Podman Compose..."
if command -v podman-compose &> /dev/null; then
    podman-compose up --build -d
elif podman compose --help &> /dev/null; then
    podman compose up --build -d
else
    echo "Error: Neither podman-compose nor 'podman compose' found."
    exit 1
fi
echo "Helios is running at http://localhost:5173"
