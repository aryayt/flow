# Flow Status Dashboard Demo
# Generates: gifs/flow-status.gif

Output gifs/flow-status.gif

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

# Full status
Type "flow status"
Sleep 500ms
Enter
Sleep 4s

# Mobile/compact status
Type "flow status --mobile"
Sleep 500ms
Enter
Sleep 3s

# Show help
Type "flow status --help"
Sleep 500ms
Enter
Sleep 2s
