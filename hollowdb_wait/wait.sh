#!/bin/sh

# Connection parameters
TARGET=${TARGET:-"http://localhost:3030"}

# How long to sleep (seconds) before each attempt.
SLEEPS=${SLEEPS:-2}

echo "Polling $TARGET"

while true; do
  curl --fail --silent "$TARGET/state"

  # check if exit code of curl is 0
  if [ $? -eq 0 ]; then
    echo "HollowDB API is ready!"
    exit 0
  fi

  sleep "$SLEEPS"
done
