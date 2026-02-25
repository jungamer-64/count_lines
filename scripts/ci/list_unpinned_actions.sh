#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <workflow-file>" >&2
  exit 2
fi

file="$1"
if [ ! -f "$file" ]; then
  echo "File not found: $file" >&2
  exit 2
fi

echo "Scanning $file for unpinned actions (shows suggestions to resolve full commit SHA)..."

# Use a grouped command so the `|| true` doesn't short-circuit the pipe
(grep -n "uses:" "$file" || true) | while IFS= read -r line; do
  lineno="$(echo "$line" | cut -d: -f1)"
  content="$(echo "$line" | cut -d: -f2-)"
  # extract the part after 'uses:' so we get owner/repo@ref
  use="$(echo "$content" | sed -E 's/.*uses:[[:space:]]*//; s/^[[:space:]]*//')"

  if [[ "$use" =~ ^([a-zA-Z0-9_.-]+)/([a-zA-Z0-9_.-]+)@(.+)$ ]]; then
    owner="${BASH_REMATCH[1]}"
    repo="${BASH_REMATCH[2]}"
    ref="${BASH_REMATCH[3]}"
    # skip if already a 40-char hex SHA
    if [[ ! "$ref" =~ ^[0-9a-f]{40}$ ]]; then
      echo "Line $lineno: uses: $owner/$repo@$ref"
      echo "  Command to resolve full SHA (requires GITHUB_TOKEN in env):"
      echo "    curl -s -H \"Accept: application/vnd.github+json\" -H \"Authorization: Bearer \$GITHUB_TOKEN\" \"https://api.github.com/repos/$owner/$repo/commits/$ref\" | jq -r .sha"
      echo
    fi
  fi
done

echo "Done. Use the above commands to fetch full commit SHAs and replace the ref in the workflow files (e.g. owner/repo@<sha>)."
