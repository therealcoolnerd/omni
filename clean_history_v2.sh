#!/bin/bash

echo "🧹 Comprehensively cleaning Claude references from git history..."

# Set the warning suppression
export FILTER_BRANCH_SQUELCH_WARNING=1

# Create a more comprehensive message filter
git filter-branch -f --msg-filter '
    # Remove both Claude lines and any empty lines that result
    sed -e "/🤖 Generated with \[Claude Code\]/d" \
        -e "/Co-Authored-By: Claude <noreply@anthropic.com>/d" \
        -e "/^[[:space:]]*$/N;/^\n$/d" \
        -e "/^[[:space:]]*$/{N;/^[[:space:]]*\n[[:space:]]*$/d;}"
' --prune-empty --tag-name-filter cat -- --branches --tags

echo "✅ Comprehensive git history cleaning completed!"
echo "📝 All Claude and Anthropic references have been removed"

# Clean up backup refs
rm -rf .git/refs/original/
git reflog expire --expire=now --all
git gc --prune=now --aggressive

echo "🗑️  Cleaned up git references and optimized repository"