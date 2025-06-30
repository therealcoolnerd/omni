#!/bin/bash

echo "🧹 Manually cleaning Claude references from recent commits..."

# Create a temporary script to edit commit messages
cat > /tmp/edit_commit.sh << 'EOF'
#!/bin/bash
# Remove Claude references from commit message
sed -i '/🤖 Generated with \[Claude Code\]/d; /Co-Authored-By: Claude <noreply@anthropic.com>/d; /^$/N;/^\n$/d' "$1"
EOF

chmod +x /tmp/edit_commit.sh

# Set the editor for git rebase
export GIT_EDITOR="/tmp/edit_commit.sh"

# Get the commits that contain Claude references
COMMITS=$(git log --grep="Claude" --grep="claude" --grep="anthropic" -i --pretty=format:"%H" | tac)

echo "Found commits with Claude references:"
echo "$COMMITS"

for commit in $COMMITS; do
    echo "Cleaning commit: $commit"
    
    # Get the current commit message
    current_msg=$(git log --format="%B" -n 1 $commit)
    
    # Clean the message
    clean_msg=$(echo "$current_msg" | sed '/🤖 Generated with \[Claude Code\]/d; /Co-Authored-By: Claude <noreply@anthropic.com>/d; /^$/N;/^\n$/d')
    
    # Create a new commit with the clean message
    git checkout $commit
    git commit --amend -m "$clean_msg"
done

echo "✅ Manual cleaning completed!"
rm /tmp/edit_commit.sh