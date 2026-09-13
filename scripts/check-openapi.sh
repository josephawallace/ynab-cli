#!/usr/bin/env sh
set -eu
snapshot="openapi/ynab-v1.86.0.yaml"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
curl --fail --silent --show-error https://api.ynab.com/papi/open_api_spec.yaml -o "$tmp"
if ! cmp -s "$snapshot" "$tmp"; then
  printf '%s\n' "Upstream OpenAPI specification changed. Review the diff and update the versioned snapshot, models, services, endpoint fixtures, and changelog together." >&2
  exit 1
fi
