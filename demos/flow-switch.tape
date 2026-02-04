# Flow Switch Demo - Fuzzy Project Switching
# Generates: gifs/flow-switch.gif

Output gifs/flow-switch.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd ~/Projects"
Enter
Sleep 500ms
Show

Sleep 1s

# Launch fuzzy finder
Type "flow switch"
Sleep 500ms
Enter
Sleep 2s

# Type to filter
Type "flow"
Sleep 1s

# Move selection
Down
Sleep 500ms
Down
Sleep 500ms
Up
Sleep 500ms

# Select (Enter)
Enter
Sleep 2s

# Show where we are
Type "pwd"
Enter
Sleep 2s

# Switch with initial query
Type "flow switch api"
Sleep 500ms
Enter
Sleep 2s

# Cancel this one
Escape
Sleep 1s
