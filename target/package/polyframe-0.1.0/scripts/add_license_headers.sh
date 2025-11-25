#!/usr/bin/env bash
set -euo pipefail

# SPDX License Header Script
# Adds Apache-2.0 license headers to all Rust source files

HEADER='// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.'

echo "Adding SPDX license headers to Rust source files..."

count=0
# Use find if not in a git repository, otherwise use git ls-files
if git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
  files=$(git ls-files '*.rs')
else
  files=$(find . -name '*.rs' -not -path './target/*' -not -path './.git/*')
fi

for f in $files; do
  # Check if file already has SPDX header
  if ! head -n 2 "$f" | grep -q "SPDX-License-Identifier"; then
    echo "Adding header to: $f"
    # Create temporary file with header and original content
    {
      echo "$HEADER"
      echo ""
      cat "$f"
    } > "$f.new"
    # Replace original file
    mv "$f.new" "$f"
    ((count++))
  fi
done

echo "Added SPDX headers to $count file(s)"
echo "Done!"

