#!/bin/bash
docker build -t movies-db-service -f docker/Dockerfile.movies-db-service .
docker build -t movies-db-ui -f docker/Dockerfile.movies-db-ui .