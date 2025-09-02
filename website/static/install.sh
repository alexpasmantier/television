#!/bin/bash
set -e

# Television Installation Script
# Automatically detects your system and installs using the best available method

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS and architecture
detect_system() {
    OS=""
    ARCH=""
    
    # Detect OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        OS="windows"
    else
        error "Unsupported OS: $OSTYPE"
    fi
    
    # Detect architecture
    case $(uname -m) in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        armv7*) ARCH="armv7" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
    
    info "Detected system: $OS ($ARCH)"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install on macOS
install_macos() {
    if command_exists brew; then
        info "Installing Television via Homebrew..."
        brew install television
        success "Television installed successfully via Homebrew!"
    elif command_exists port; then
        warning "MacPorts detected but Television may not be available. Falling back to binary download..."
        install_binary
    else
        warning "No package manager found. Installing via binary download..."
        install_binary
    fi
}

# Install on Linux
install_linux() {
    # Try package managers in order of preference
    if command_exists nix; then
        info "Installing Television via Nix..."
        nix profile install nixpkgs#television
        success "Television installed successfully via Nix!"
    elif command_exists pacman && [[ -f /etc/arch-release ]]; then
        info "Installing Television via pacman (Arch Linux)..."
        sudo pacman -S --noconfirm television
        success "Television installed successfully via pacman!"
    elif command_exists apk && [[ -f /etc/chimera-release ]]; then
        info "Installing Television via apk (Chimera Linux)..."
        sudo apk add chimera-repo-user
        sudo apk add television
        success "Television installed successfully via apk!"
    elif command_exists apt-get && [[ -f /etc/debian_version ]]; then
        info "Installing Television via .deb package (Debian-based)..."
        install_deb
    elif command_exists dnf; then
        warning "Fedora/RHEL detected but no native package available. Installing via binary..."
        install_binary
    elif command_exists zypper; then
        warning "openSUSE detected but no native package available. Installing via binary..."
        install_binary
    elif command_exists cargo; then
        warning "No native package available. Installing via Cargo..."
        install_cargo
    else
        warning "No supported package manager found. Installing via binary download..."
        install_binary
    fi
}

# Install .deb package on Debian-based systems
install_deb() {
    # Detect architecture-specific package
    case "$ARCH" in
        x86_64) DEB_ARCH="x86_64-unknown-linux-musl" ;;
        aarch64) DEB_ARCH="aarch64-unknown-linux-gnu" ;;
        *) 
            warning "No .deb package for $ARCH architecture. Falling back to binary..."
            install_binary
            return
            ;;
    esac
    
    info "Fetching latest release information..."
    VER=$(curl -s "https://api.github.com/repos/alexpasmantier/television/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    
    if [[ -z "$VER" ]]; then
        error "Failed to fetch latest version. Please check your internet connection."
    fi
    
    info "Downloading Television $VER for $DEB_ARCH..."
    DEB_FILE="tv-$VER-$DEB_ARCH.deb"
    
    if ! curl -LO "https://github.com/alexpasmantier/television/releases/download/$VER/$DEB_FILE"; then
        error "Failed to download .deb package"
    fi
    
    info "Installing .deb package..."
    sudo dpkg -i "$DEB_FILE"
    
    # Clean up
    rm -f "$DEB_FILE"
    
    success "Television $VER installed successfully!"
}

# Install via Cargo
install_cargo() {
    if ! command_exists rustc; then
        info "Rust not found. Installing Rust first..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    info "Installing Television via Cargo..."
    cargo install --locked television
    success "Television installed successfully via Cargo!"
}

# Install pre-compiled binary
install_binary() {
    info "Installing Television via pre-compiled binary..."
    
    # Map architecture to release naming
    case "$OS-$ARCH" in
        linux-x86_64) BINARY_TARGET="x86_64-unknown-linux-musl" ;;
        linux-aarch64) BINARY_TARGET="aarch64-unknown-linux-gnu" ;;
        macos-x86_64) BINARY_TARGET="x86_64-apple-darwin" ;;
        macos-aarch64) BINARY_TARGET="aarch64-apple-darwin" ;;
        *) error "No pre-compiled binary available for $OS-$ARCH" ;;
    esac
    
    info "Fetching latest release information..."
    VER=$(curl -s "https://api.github.com/repos/alexpasmantier/television/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    
    if [[ -z "$VER" ]]; then
        error "Failed to fetch latest version. Please check your internet connection."
    fi
    
    DIRNAME="tv-$VER-$BINARY_TARGET"
    TARBALL="$DIRNAME.tar.gz"
    URL="https://github.com/alexpasmantier/television/releases/download/$VER/$TARBALL"
    
    info "Downloading Television $VER for $OS-$ARCH..."
    if ! curl -LO "$URL"; then
        error "Failed to download binary package"
    fi
    
    info "Extracting binary..."
    tar -xzf "$TARBALL"
    
    # Install to appropriate location
    if [[ "$OS" == "macos" ]]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="/usr/local/bin"
    fi
    
    info "Installing to $INSTALL_DIR..."
    sudo mkdir -p "$INSTALL_DIR"
    sudo mv "$DIRNAME/tv" "$INSTALL_DIR/tv"
    sudo chmod +x "$INSTALL_DIR/tv"
    
    # Clean up
    rm -rf $DIRNAME*
    
    success "Television $VER installed successfully to $INSTALL_DIR/tv!"
}

# Install on Windows (Git Bash/MSYS2/WSL)
install_windows() {
    if command_exists winget.exe; then
        info "Installing Television via WinGet..."
        winget.exe install --exact --id alexpasmantier.television
        success "Television installed successfully via WinGet!"
    elif command_exists scoop; then
        info "Installing Television via Scoop..."
        scoop bucket add extras
        scoop install television
        success "Television installed successfully via Scoop!"
    else
        error "No supported Windows package manager found. Please install WinGet or Scoop first."
    fi
}

# Main installation logic
main() {
    echo "🔍 Television Installation Script"
    echo "================================"
    
    detect_system
    
    case "$OS" in
        linux) install_linux ;;
        macos) install_macos ;;
        windows) install_windows ;;
        *) error "Unsupported operating system: $OS" ;;
    esac
    
    echo ""
    info "Installation complete! You can now use the 'tv' command."
    info "Run 'tv --help' to get started."
    info "Documentation: https://github.com/alexpasmantier/television"
}

# Handle Ctrl+C
trap 'echo ""; error "Installation cancelled by user"' INT

# Run main function
main "$@"
