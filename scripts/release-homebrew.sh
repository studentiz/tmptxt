#!/usr/bin/env bash
# Release a new tmptxt version: tag the repo, then update the Homebrew tap
# formula so `brew install studentiz/tap/tmptxt` gets the new version.
#
# Prerequisites: gh CLI authed with 'repo' scope, write access to
#   github.com/studentiz/tmptxt and github.com/studentiz/homebrew-tap.
#
# Usage:
#   scripts/release-homebrew.sh v0.2.0
#
# Bump the version in Cargo.toml and commit it BEFORE running this script.
# The tap checkout lives in $TAP_DIR (default /tmp/homebrew-tap); if it is
# not a git checkout yet, the script clones it there.
set -euo pipefail

RELEASE_TAG="${1:?usage: release-homebrew.sh <tag> e.g. v0.2.0}"
TAP_DIR="${TAP_DIR:-/tmp/homebrew-tap}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# --- Sanity checks ---------------------------------------------------------
command -v gh >/dev/null 2>&1 || { echo "Install GitHub CLI first: https://cli.github.com/" >&2; exit 1; }
if ! gh auth status >/dev/null 2>&1; then
  echo "Not logged in. Run: gh auth login" >&2
  exit 1
fi

CARGO_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)"
if [[ "$RELEASE_TAG" != "v$CARGO_VERSION" ]]; then
  echo "Tag '$RELEASE_TAG' does not match Cargo.toml version '$CARGO_VERSION'." >&2
  echo "Bump Cargo.toml and commit first, or pass the matching tag." >&2
  exit 1
fi

# --- Tag and push the release ---------------------------------------------
if git rev-parse -q --verify "refs/tags/$RELEASE_TAG" >/dev/null 2>&1; then
  echo "Tag $RELEASE_TAG already exists locally; pushing it."
else
  git tag "$RELEASE_TAG"
fi
git push origin "$RELEASE_TAG"

# --- Compute the source-tarball sha256 ------------------------------------
# GitHub generates archives lazily; retry a few times if it is not ready yet.
TARBALL_URL="https://github.com/studentiz/tmptxt/archive/refs/tags/${RELEASE_TAG}.tar.gz"
SHA256=""
for _ in {1..10}; do
  SHA256="$(curl -fsSL "$TARBALL_URL" 2>/dev/null | shasum -a 256 | awk '{print $1}')" || true
  if [[ "$SHA256" =~ ^[0-9a-f]{64}$ ]]; then
    break
  fi
  echo "Source tarball not ready yet, retrying..." >&2
  sleep 3
done
if [[ ! "$SHA256" =~ ^[0-9a-f]{64}$ ]]; then
  echo "Could not fetch a valid tarball from $TARBALL_URL" >&2
  exit 1
fi
echo "sha256($RELEASE_TAG): $SHA256"

# --- Update the Homebrew tap formula ---------------------------------------
if [[ ! -d "$TAP_DIR/.git" ]]; then
  git clone -q https://github.com/studentiz/homebrew-tap.git "$TAP_DIR"
fi
cd "$TAP_DIR"
git pull -q --ff-only origin main 2>/dev/null || true

cat > "$TAP_DIR/Formula/tmptxt.rb" <<EOF
class Tmptxt < Formula
  desc "Minimal auto-saving terminal scratchpad (Rust)"
  homepage "https://github.com/studentiz/tmptxt"
  url "https://github.com/studentiz/tmptxt/archive/refs/tags/${RELEASE_TAG}.tar.gz"
  sha256 "${SHA256}"
  version "$CARGO_VERSION"
  license "Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "tmptxt", shell_output("#{bin}/tmptxt --version")
  end
end
EOF

git add Formula/tmptxt.rb
git -c user.name="studentiz" -c user.email="studentiz@users.noreply.github.com" \
  commit -q -m "Update tmptxt to ${RELEASE_TAG}"
git push -q origin main

echo "Done. Formula pushed to https://github.com/studentiz/homebrew-tap"
echo "Verify: brew install studentiz/tap/tmptxt"
