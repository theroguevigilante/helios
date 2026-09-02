#!/bin/bash
echo "Starting Helios via Docker Compose..."
docker compose up --build -d
echo "Helios is running at http://localhost:5173"
