#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${CYAN}"
    echo "╭─────────────────────────────────────────────╮"
    echo "│             Flow Installer                  │"
    echo "│   Git Worktree + Tmux Workflow Manager      │"
    echo "╰─────────────────────────────────────────────╯"
    echo -e "${NC}"
}

print_step() {
    echo -e "${BLUE}▶${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux" ;;
        Darwin*)    OS="macos" ;;
        CYGWIN*|MINGW*|MSYS*) OS="windows" ;;
        *)          OS="unknown" ;;
    esac
    echo "$OS"
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   ARCH="x86_64" ;;
        aarch64|arm64)  ARCH="arm64" ;;
        armv7l|armv6l)  ARCH="arm" ;;
        *)              ARCH="unknown" ;;
    esac
    echo "$ARCH"
}

detect_package_manager() {
    if command -v brew &> /dev/null; then
        echo "brew"
    elif command -v apt-get &> /dev/null; then
        echo "apt"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v apk &> /dev/null; then
        echo "apk"
    else
        echo "unknown"
    fi
}

install_prerequisites() {
    local pkg_manager=$1

    print_step "Installing prerequisites (git, tmux)..."

    case "$pkg_manager" in
        brew)
            brew install git tmux 2>/dev/null || true
            ;;
        apt)
            sudo apt-get update -qq
            sudo apt-get install -y git tmux
            ;;
        dnf)
            sudo dnf install -y git tmux
            ;;
        pacman)
            sudo pacman -S --noconfirm --needed git tmux
            ;;
        apk)
            sudo apk add git tmux
            ;;
        *)
            print_warning "Could not detect package manager. Please install git and tmux manually."
            return 1
            ;;
    esac

    print_success "Prerequisites installed"
}

check_prerequisites() {
    local missing=""

    if ! command -v git &> /dev/null; then
        missing="$missing git"
    fi

    if ! command -v tmux &> /dev/null; then
        missing="$missing tmux"
    fi

    echo "$missing"
}

install_rust() {
    if command -v cargo &> /dev/null; then
        print_success "Rust is already installed ($(rustc --version 2>/dev/null | cut -d' ' -f2))"
        return 0
    fi

    print_step "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    # Source cargo env for this session
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi

    print_success "Rust installed successfully"
}

install_flow() {
    print_step "Installing flow-cli from crates.io..."
    cargo install flow-cli
    print_success "Flow installed successfully"
}

print_header

OS=$(detect_os)
ARCH=$(detect_arch)
PKG_MANAGER=$(detect_package_manager)

echo -e "${BOLD}System detected:${NC}"
echo "  OS:           $OS"
echo "  Architecture: $ARCH"
echo "  Package mgr:  $PKG_MANAGER"
echo ""

# Handle Windows/WSL
if [ "$OS" = "windows" ]; then
    print_warning "Native Windows detected!"
    echo ""
    echo "Flow requires tmux, which is not available natively on Windows."
    echo "Please use one of these options:"
    echo ""
    echo "  1. ${BOLD}WSL2 (Recommended)${NC}"
    echo "     Install WSL2: wsl --install"
    echo "     Then run this script inside WSL2"
    echo ""
    echo "  2. ${BOLD}Git Bash + Windows Terminal${NC}"
    echo "     Limited functionality (no tmux sessions)"
    echo ""
    exit 1
fi

if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
    print_error "Unsupported platform: $(uname -s) / $(uname -m)"
    exit 1
fi

# Check and install prerequisites
MISSING=$(check_prerequisites)
if [ -n "$MISSING" ]; then
    print_warning "Missing prerequisites:$MISSING"

    if [ "$PKG_MANAGER" != "unknown" ]; then
        read -p "Install missing prerequisites? [Y/n] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            install_prerequisites "$PKG_MANAGER"
        else
            print_error "Prerequisites required. Please install:$MISSING"
            exit 1
        fi
    else
        print_error "Please install manually:$MISSING"
        exit 1
    fi
else
    print_success "Prerequisites satisfied (git, tmux)"
fi

# Install Rust
install_rust

# Install Flow
install_flow

# Verify installation
echo ""
echo -e "${GREEN}╭─────────────────────────────────────────────╮${NC}"
echo -e "${GREEN}│          Installation Complete!             │${NC}"
echo -e "${GREEN}╰─────────────────────────────────────────────╯${NC}"
echo ""
flow --version 2>/dev/null || echo "flow-cli installed"
echo ""
echo "Get started:"
echo "  ${CYAN}flow status${NC}        - Show dashboard"
echo "  ${CYAN}flow branch <name>${NC} - Create worktree + tmux session"
echo "  ${CYAN}flow switch${NC}        - Fuzzy-find projects"
echo ""
echo "Documentation: https://github.com/aryayt/flow"
