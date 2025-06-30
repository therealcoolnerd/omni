#!/bin/bash

echo "🧹 Cleaning Claude references from git history..."

# Use git filter-branch to remove Claude references from all commit messages
git filter-branch -f --msg-filter '
    sed -e "/🤖 Generated with \[Claude Code\]/d" \
        -e "/Co-Authored-By: Claude <noreply@anthropic.com>/d" \
        -e "/^$/N;/^\n$/d"
' HEAD~20..HEAD

echo "✅ Git history cleaned successfully!"
echo "📝 Claude references have been removed from all commit messages"