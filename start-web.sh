#!/bin/sh
# TODO: This won't properly exit yet!
cd /app
Xvfb&
xvfb-run ./web