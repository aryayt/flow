# Flow CLI - Main Demo
# Generates: gifs/demo.gif

Output gifs/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

# Setup - ensure we're in a git repo
Hide
Type "cd ~/Projects/flow"
Enter
Sleep 500ms
Show

# Title
Sleep 1s
Type "# Flow - Git Worktree + Tmux Workflow Manager"
Sleep 500ms
Enter
Sleep 1s

# Show status dashboard
Type "flow status"
Sleep 500ms
Enter
Sleep 3s

# Create a new branch with worktree
Type "flow branch feature/demo"
Sleep 500ms
Enter
Sleep 2s

# List worktrees
Type "flow worktree list"
Sleep 500ms
Enter
Sleep 2s

# Show the fuzzy finder (we'll cancel it)
Type "flow switch"
Sleep 500ms
Enter
Sleep 2s

# Exit the fuzzy finder
Type "q"
Sleep 1s

# Clean up the demo worktree
Type "flow worktree remove feature/demo"
Sleep 500ms
Enter
Sleep 1s
Type "y"
Enter
Sleep 2s

# Final message
Type "# That's Flow! Ship faster with isolated worktrees."
Enter
Sleep 2s
