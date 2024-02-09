#!/bin/sh

# Connection parameters
TARGET=${TARGET:-"http://localhost:3000"}

# How long to sleep (seconds) before each attempt.
SLEEPS=${SLEEPS:-2}

echo "Polling $TARGET"

while true; do
  # with --fail, it will fail for an error code >=400
  # our micro API returns 503 when it is still refreshing
  curl --fail --output /dev/null --silent --data '{"route": "STATE"}' "$TARGET"

  # check if exit code of curl is 0
  if [ $? -eq 0 ]; then
    echo "HollowDB container is ready!"
    exit 0
  fi

  sleep "$SLEEPS"
done
