#!/bin/bash
# scripts/teardown.sh
echo "Teardown Regtest Environment"
docker-compose down -v
