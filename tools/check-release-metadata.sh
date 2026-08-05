#!/usr/bin/env bash
set -euo pipefail

metadata="$(mktemp "${TMPDIR:-/tmp}/mcp-as-release-metadata.XXXXXX.json")"
trap 'rm -f "$metadata"' EXIT
cargo metadata --locked --no-deps --format-version 1 >"$metadata"

version="$(jq -r '.packages[0].version' "$metadata")"

jq -e '
  [.packages[].version] | unique | length == 1
' "$metadata" >/dev/null
jq -e '
  all(.packages[];
    .version == $version and
    .license == "MIT OR Apache-2.0" and
    .repository == "https://github.com/KeishiS/mcp-authorization-server" and
    .publish == [])
' --arg version "$version" "$metadata" >/dev/null

test -s LICENSE-MIT
test -s LICENSE-APACHE

release_tag="${MCP_AUTHORIZATION_SERVER_RELEASE_TAG:-}"
if [[ -n "$release_tag" ]]; then
  if [[ "$release_tag" != "v$version" ]]; then
    echo "release tagとworkspace版が一致しません: $release_tag != v$version" >&2
    exit 1
  fi
  if ! grep -Eq "^== ${version} - [0-9]{4}-[0-9]{2}-[0-9]{2}$" CHANGELOG.adoc; then
    echo "CHANGELOG.adocの${version}へ公開日を記録してください" >&2
    exit 1
  fi
else
  grep -Eq "^== ${version} - (未公開|[0-9]{4}-[0-9]{2}-[0-9]{2})$" CHANGELOG.adoc
fi

echo "release metadataを確認しました: v${version}"
