#!/usr/bin/env bash
set -euo pipefail

metadata="$(mktemp "${TMPDIR:-/tmp}/mcp-as-release-metadata.XXXXXX.json")"
trap 'rm -f "$metadata"' EXIT
cargo metadata --locked --no-deps --format-version 1 >"$metadata"

jq -e '
  [.packages[].version] | unique | length == 1
' "$metadata" >/dev/null
jq -e '
  all(.packages[];
    .version == "0.1.0" and
    .license == "MIT OR Apache-2.0" and
    .repository == "https://github.com/KeishiS/mcp-authorization-server" and
    .publish == [])
' "$metadata" >/dev/null

test -s LICENSE-MIT
test -s LICENSE-APACHE
grep -Fq "== 0.1.0 - 未公開" CHANGELOG.adoc

echo "release metadataを確認しました"
