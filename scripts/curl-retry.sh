#!/usr/bin/env bash
# Download a URL with exponential backoff (handles transient 502/504).
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: curl-retry.sh <url> <output-file>" >&2
  exit 1
fi

url="$1"
out="$2"
delays=(4 8 16 32)

for i in "${!delays[@]}"; do
  attempt=$((i + 1))
  if curl -fsSL --retry 2 --retry-delay 2 -o "$out" "$url"; then
    exit 0
  fi
  echo "curl failed for $url (attempt ${attempt}/${#delays[@]})" >&2
  if [[ $attempt -lt ${#delays[@]} ]]; then
    sleep "${delays[$i]}"
  fi
done

echo "curl-retry: giving up on $url" >&2
exit 1
