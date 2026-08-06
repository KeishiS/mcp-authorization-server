#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "$0")/.." && pwd)"
checker="$repository_root/tools/verify-pre-tag-release-gate.sh"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/mcp-as-release-gate.XXXXXX")"
trap 'rm -rf "$test_root"' EXIT

git -C "$test_root" init -q -b main
git -C "$test_root" config user.name "Release gate test"
git -C "$test_root" config user.email "release-gate-test@example.invalid"
git -C "$test_root" commit -q --allow-empty -m "test commit"
commit="$(git -C "$test_root" rev-parse HEAD)"

mkdir -p "$test_root/bin"
cat >"$test_root/bin/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
test "$1" = api
test "$2" = "repos/example/repository/actions/runs/123456789"
printf '%s\n' "$MOCK_RUN_JSON"
EOF
chmod +x "$test_root/bin/gh"

run_json() {
  local title="$1"
  local sha="$2"
  local conclusion="$3"
  jq -cn \
    --arg title "$title" \
    --arg sha "$sha" \
    --arg conclusion "$conclusion" '
      {
        name: "Release gate",
        display_title: $title,
        event: "workflow_dispatch",
        head_branch: "main",
        head_sha: $sha,
        status: "completed",
        conclusion: $conclusion,
        html_url: "https://example.invalid/actions/runs/123456789"
      }
    '
}

check() {
  local release_tag="$1"
  local run="$2"
  (
    cd "$test_root"
    PATH="$test_root/bin:$PATH" \
      GITHUB_REPOSITORY=example/repository \
      GH_TOKEN=test-token \
      MOCK_RUN_JSON="$run" \
      bash "$checker" "$release_tag"
  )
}

expect_failure() {
  local description="$1"
  local release_tag="$2"
  local run="$3"
  if check "$release_tag" "$run" >/dev/null 2>&1; then
    echo "失敗すべき検査が成功しました: $description" >&2
    exit 1
  fi
}

git -C "$test_root" tag -a v1.0.0 -m "v1.0.0" -m "Release-Gate-Run: 123456789"
check v1.0.0 "$(run_json "Release gate: v1.0.0" "$commit" success)" >/dev/null

git -C "$test_root" tag v1.0.1
expect_failure \
  "lightweight tag" \
  v1.0.1 \
  "$(run_json "Release gate: v1.0.1" "$commit" success)"

git -C "$test_root" tag -a v1.0.2 -m "v1.0.2"
expect_failure \
  "Run IDの欠落" \
  v1.0.2 \
  "$(run_json "Release gate: v1.0.2" "$commit" success)"

git -C "$test_root" tag -a v1.0.3 -m "v1.0.3" -m "Release-Gate-Run: 123456789"
expect_failure \
  "tag名の不一致" \
  v1.0.3 \
  "$(run_json "Release gate: v9.9.9" "$commit" success)"

git -C "$test_root" tag -a v1.0.4 -m "v1.0.4" -m "Release-Gate-Run: 123456789"
expect_failure \
  "commit SHAの不一致" \
  v1.0.4 \
  "$(run_json "Release gate: v1.0.4" "0000000000000000000000000000000000000000" success)"

git -C "$test_root" tag -a v1.0.5 -m "v1.0.5" -m "Release-Gate-Run: 123456789"
expect_failure \
  "未成功のrun" \
  v1.0.5 \
  "$(run_json "Release gate: v1.0.5" "$commit" failure)"

echo "事前Release gate検査の回帰試験に成功しました"
