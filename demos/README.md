# Flow Terminal Demos

This directory contains VHS tape files for generating terminal demo GIFs.

## Prerequisites

Install VHS (terminal recording tool):

```bash
# macOS
brew install vhs

# Linux (via Go)
go install github.com/charmbracelet/vhs@latest

# Or download from releases
# https://github.com/charmbracelet/vhs/releases
```

VHS also requires `ffmpeg` and `ttyd`:

```bash
# macOS
brew install ffmpeg ttyd

# Ubuntu/Debian
sudo apt install ffmpeg
# ttyd: download from https://github.com/tsl0922/ttyd/releases
```

## Generating GIFs

Generate all demos:

```bash
cd demos
for tape in *.tape; do
    vhs "$tape"
done
```

Or generate a specific demo:

```bash
vhs demo.tape
```

GIFs are output to the `gifs/` directory.

## Tape Files

| File | Description |
|------|-------------|
| `demo.tape` | Main hero demo showing full workflow |
| `flow-status.tape` | Status dashboard |
| `flow-branch.tape` | Branch/worktree creation |
| `flow-switch.tape` | Fuzzy project switching |

## Customization

Common settings used across all tapes:

```tape
Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
```

See the [VHS documentation](https://github.com/charmbracelet/vhs) for more options.

## GitHub Actions

The `demos.yml` workflow automatically regenerates GIFs when tape files change.
To manually trigger regeneration, use the workflow dispatch.
