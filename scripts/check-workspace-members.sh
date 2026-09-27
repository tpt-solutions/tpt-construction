#!/usr/bin/env bash
# Fails when a crates/* directory exists but is not listed in [workspace]
# members of the root Cargo.toml (and vice versa), so crate drift cannot
# recur. Run in CI and locally: bash scripts/check-workspace-members.sh
set -euo pipefail
cd "$(dirname "$0")/.."

members=$(sed -n '/^members = \[/,/^[[:space:]]*\]/p' Cargo.toml)
failed=0

for dir in crates/*/; do
    name=$(basename "$dir")
    if ! grep -q "\"crates/$name\"" <<<"$members"; then
        echo "ERROR: crates/$name exists but is not listed in [workspace] members" >&2
        failed=1
    fi
done

while read -r entry; do
    if [ ! -d "$entry" ]; then
        echo "ERROR: [workspace] members lists \"$entry\" but the directory does not exist" >&2
        failed=1
    fi
done < <(grep -o '"crates/[^"]*"' <<<"$members" | tr -d '"')

if [ "$failed" -ne 0 ]; then
    echo "" >&2
    echo "Fix the root Cargo.toml members list (or remove the stray directory)." >&2
    exit 1
fi
echo "workspace membership check: every crates/ directory is a member and every member exists"
