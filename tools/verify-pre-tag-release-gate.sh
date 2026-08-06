#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:-}"
if [[ -z "$release_tag" ]]; then
  echo "検査するrelease tagを指定してください" >&2
  exit 1
fi
if [[ -z "${GITHUB_REPOSITORY:-}" || -z "${GH_TOKEN:-}" ]]; then
  echo "GITHUB_REPOSITORYとGH_TOKENが必要です" >&2
  exit 1
fi

tag_ref="refs/tags/$release_tag"
if [[ "$(git cat-file -t "$tag_ref")" != tag ]]; then
  echo "$release_tagは注釈付きtagではありません" >&2
  exit 1
fi

tagged_commit="$(git rev-list -n 1 "$tag_ref")"
gate_run_id="$(
  git for-each-ref --format='%(contents)' "$tag_ref" |
    sed -n 's/^Release-Gate-Run: \([0-9][0-9]*\)$/\1/p' || true
)"
if [[ ! "$gate_run_id" =~ ^[0-9]+$ ]]; then
  echo "$release_tagの注釈にRelease-Gate-Runを1件記録してください" >&2
  exit 1
fi

run="$(gh api "repos/$GITHUB_REPOSITORY/actions/runs/$gate_run_id")"
run_url="$(
  jq -er \
    --arg sha "$tagged_commit" \
    --arg title "Release gate: $release_tag" '
      select(
        .name == "Release gate" and
        .display_title == $title and
        .event == "workflow_dispatch" and
        .head_branch == "main" and
        .head_sha == $sha and
        .status == "completed" and
        .conclusion == "success"
      )
      | .html_url
    ' <<<"$run" || true
)"
if [[ -z "$run_url" ]]; then
  echo "tagと一致する成功済みの事前Release gateを確認できません" >&2
  exit 1
fi

echo "事前Release gateを確認しました: $run_url"
