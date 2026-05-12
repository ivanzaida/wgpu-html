---
sidebar_position: 1
---

# Building from Source

## Prerequisites

- **Rust 1.78+** — install via [rustup](https://rustup.rs)
- **Git** — for cloning the repository
- **C compiler** — for native dependencies (cosmic-text, wgpu-hal)

## Clone and Build

```bash
git clone https://github.com/ivanzaida/lui.git
cd lui
cargo build --workspace
```

## Platform-Specific Setup

### Ubuntu / Debian

```bash
sudo apt install build-essential pkg-config libx11-dev libwayland-dev libudev-dev
```

### Fedora

```bash
sudo dnf install gcc pkg-config libX11-devel wayland-devel systemd-devel
```

### macOS

No additional dependencies needed. Xcode command-line tools must be installed:

```bash
xcode-select --install
```

### Windows

No additional dependencies needed. Visual Studio Build Tools or the Windows SDK should be installed.

## Build Commands

```bash
# Build everything
cargo build --workspace

# Release build
cargo build --workspace --release

# Build only the core library
cargo build -p lui

# Build with a specific driver
cargo build -p lui-driver-winit
```

## Development Build

For faster iteration during development:

```bash
# Debug build with default optimizations
cargo build -p lui-demo

# Run tests for all crates
cargo test --workspace

# Run tests for a specific crate
cargo test -p lui-layout-old
```
