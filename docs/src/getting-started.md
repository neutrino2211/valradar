# Getting Started

This section will guide you through setting up Valradar and running your first plugin.

## Overview

Getting Valradar up and running involves three steps:

1. **Install the Valradar binary** — The Rust-based runtime and CLI
2. **Install the Python SDK** — For plugin development
3. **Run a plugin** — Test your installation with an example

## Quick Setup

If you're in a hurry, here's the fastest path:

```bash
# Option 1: Download pre-built binary (recommended)
# Visit https://github.com/neutrino2211/valradar/releases
# Download the binary for your platform

# Option 2: Build from source
git clone https://github.com/neutrino2211/valradar.git
cd valradar
cargo build --release

# Install Python SDK
pip install valradar

# Run a test
valradar run examples.emails https://example.com
```

## What's Next

- [Installation](./installation.md) — Detailed installation instructions for all platforms
- [Quick Start](./quick-start.md) — Run your first plugin in 5 minutes

## System Requirements

| Component | Requirement |
|-----------|-------------|
| OS | Linux, macOS, Windows |
| Python | 3.10 or higher |
| Rust | 1.70+ (only for building from source) |
| Memory | 512MB minimum, 2GB recommended |

## Project Structure

After cloning or downloading Valradar, you'll see this structure:

```
valradar/
├── src/              # Rust source code
├── python/           # Python SDK
│   └── valradar/     # SDK package
├── examples/         # Example plugins
│   └── emails.py     # Email extraction plugin
├── modules/          # Plugin discovery paths
├── Cargo.toml        # Rust dependencies
└── README.md
```

The `examples/` directory contains ready-to-use plugins that demonstrate Valradar's capabilities. You can also place your own plugins in `modules/` for automatic discovery.
