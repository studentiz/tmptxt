#!/usr/bin/env bash
# Create github.com/studentiz/tmptxt (if missing) and push the current branch.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v gh >/dev/null 2>&1; then
  echo "Install GitHub CLI first: https://cli.github.com/"
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "Not logged in. Run: gh auth login"
  exit 1
fi

if git remote get-url origin >/dev/null 2>&1; then
  echo "Remote 'origin' is already set. Pushing current branch..."
  git push -u origin "$(git branch --show-current)"
else
  gh repo create tmptxt --public \
    --description "Minimal auto-saving terminal scratchpad (Rust)" \
    --source=. --remote=origin --push
fi

echo "Done. Repo: https://github.com/studentiz/tmptxt"
