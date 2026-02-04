# Flow Branch Demo - Creating Worktrees
# Generates: gifs/flow-branch.gif

Output gifs/flow-branch.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd ~/Projects/flow"
Enter
Sleep 500ms
Show

Sleep 1s

# Show current worktrees
Type "flow worktree list"
Sleep 500ms
Enter
Sleep 2s

# Create a feature branch
Type "flow branch feature/auth"
Sleep 500ms
Enter
Sleep 2s

# List again to show new worktree
Type "flow worktree list"
Sleep 500ms
Enter
Sleep 2s

# Create from a specific base
Type "flow branch hotfix/urgent --base v1.0.0"
Sleep 500ms
Enter
Sleep 2s

# Show all worktrees
Type "flow worktree list"
Sleep 500ms
Enter
Sleep 3s

# Clean up
Hide
Type "flow worktree remove feature/auth"
Enter
Type "y"
Enter
Type "flow worktree remove hotfix/urgent"
Enter
Type "y"
Enter
Sleep 1s
