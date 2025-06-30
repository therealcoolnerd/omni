#!/bin/bash

# Script to clean Claude references from commit messages
# This removes the two lines:
# 🤖 Generated with [Claude Code](https://claude.ai/code)
# Co-Authored-By: Claude <noreply@anthropic.com>

# Set the EDITOR environment variable to use this script
export GIT_EDITOR="sed -i '/🤖 Generated with \[Claude Code\]/d; /Co-Authored-By: Claude <noreply@anthropic.com>/d'"

# List of commits to clean (from newest to oldest)
COMMITS=(
    "5d25ade4"
    "641e6842" 
    "ce50c343"
    "e9663362"
    "03992868"
)

echo "Cleaning Claude references from commit messages..."

# We'll use filter-branch to rewrite commit messages
git filter-branch --msg-filter '
    sed "/🤖 Generated with \[Claude Code\]/d; /Co-Authored-By: Claude <noreply@anthropic.com>/d"
' --prune-empty -- --all

echo "Commit cleaning complete!"