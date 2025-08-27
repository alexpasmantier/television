# Installation

[![Packaging status](https://repology.org/badge/vertical-allrepos/television.svg)](https://repology.org/project/television/versions)

## Quick Install (Recommended)

For an automatic installation that detects your system and uses the best available method:

```bash
curl -fsSL https://alexpasmantier.github.io/television/install.sh | bash
```

Or if you prefer to inspect the script first:

```bash
curl -fsSL https://alexpasmantier.github.io/television/install.sh -o install.sh
chmod +x install.sh
./install.sh
```

## Nix

Television is [available on `nixpkgs`](https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/te/television/package.nix)

```bash
nix run nixpkgs#television
```

## Homebrew

```bash
brew install television
```

## Scoop

```bash
scoop bucket add extras
scoop install television
```

## WinGet

```pwsh
winget install --exact --id alexpasmantier.television
```

## Arch Linux

```bash
pacman -S television
```

## Chimera Linux

```bash
apk add chimera-repo-user
apk add television
```

## Debian-based (Debian, Ubuntu, Pop!\_OS, Linux Mint, etc.)

```bash
VER=`curl -s "https://api.github.com/repos/alexpasmantier/television/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/'`
curl -LO https://github.com/alexpasmantier/television/releases/download/$VER/tv-$VER-x86_64-unknown-linux-musl.deb
echo $VER
sudo dpkg -i tv-$VER-x86_64-unknown-linux-musl.deb
```

## Conda-forge (cross-platform)

```bash
pixi global install television
```

## NetBSD (pkgsrc)

```bash
pkgin install television
```

## Pre-compiled Binary

From the [latest release](https://github.com/alexpasmantier/television/releases/latest) page:

- Download the latest release asset for your platform (e.g. `tv-vX.X.X-x86_64-unknown-linux-musl.tar.gz` if you're on a linux x86 machine)
- Unpack and copy to the relevant location on your system (e.g. `/usr/local/bin` on macos and linux for a global installation)

## Crates.io

Setup the latest stable Rust toolchain via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

Install `television`:

```bash
cargo install --locked television
```

## Building from source

If you want to benefit from the latest updates on main, clone the repo and build from source by running:

```bash
git clone git@github.com:alexpasmantier/television.git && cd television
just build release
```

You can then alias `tv` to the produced binary:

```bash
alias tv=$TELEVISION_DIR/target/release/tv
```
