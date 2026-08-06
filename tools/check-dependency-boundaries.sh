#!/usr/bin/env bash
set -euo pipefail

metadata="$(mktemp "${TMPDIR:-/tmp}/mcp-as-metadata.XXXXXX.json")"
trap 'rm -f "$metadata"' EXIT
cargo metadata --locked --no-deps --format-version 1 >"$metadata"

jq -e '
  def normal_dependencies($package):
    [.packages[] | select(.name == $package) | .dependencies[] | select(.kind == null) | .name] | sort;
  normal_dependencies("mcp-authorization-server") ==
    ["async-trait", "base64", "serde", "sha2", "url"] and
  normal_dependencies("mcp-authorization-server-cimd") ==
    ["async-trait", "mcp-authorization-server", "reqwest", "serde", "serde_json", "tokio", "tracing", "url"]
' "$metadata" >/dev/null

echo "crate間の依存境界を確認しました"
