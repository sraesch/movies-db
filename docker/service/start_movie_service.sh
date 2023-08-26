#!/bin/bash

exec /opt/movies-db/bin/movies-db-cli --root-dir /var/data/movies -l ${LOG_LEVEL:-info}