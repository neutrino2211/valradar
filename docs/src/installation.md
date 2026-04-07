# Installation

This guide covers all methods for installing Valradar.

## Pre-built Binaries (Recommended)

The easiest way to install Valradar is to download a pre-built binary from the GitHub releases page.

### Download

Visit the [Releases page](https://github.com/neutrino2211/valradar/releases) and download the appropriate binary for your platform:

| Platform | File |
|----------|------|
| Linux (x64) | `valradar-linux-x86_64` |
| macOS (x64) | `valradar-darwin-x86_64` |
| macOS (ARM) | `valradar-darwin-aarch64` |
| Windows | `valradar-windows-x86_64.exe` |

### Linux / macOS

```bash
# Download (replace with actual release URL)
curl -LO https://github.com/neutrino2211/valradar/releases/latest/download/valradar-linux-x86_64

# Make executable
chmod +x valradar-linux-x86_64

# Move to PATH (optional)
sudo mv valradar-linux-x86_64 /usr/local/bin/valradar

# Verify installation
valradar --version
```

### Windows

1. Download `valradar-windows-x86_64.exe` from the releases page
2. Rename to `valradar.exe` (optional)
3. Add the directory to your PATH, or run from the download location

```powershell
# Verify installation
.\valradar.exe --version
```

## Building from Source

If you prefer to build from source or need the latest features:

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or higher
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/neutrino2211/valradar.git
cd valradar

# Build in release mode
cargo build --release

# The binary is at ./target/release/valradar
./target/release/valradar --version
```

### Install to PATH

```bash
# Option 1: Copy to /usr/local/bin
sudo cp target/release/valradar /usr/local/bin/

# Option 2: Use cargo install (if you have a Cargo.toml)
cargo install --path .
```

## Python SDK

The Python SDK is required for writing and running plugins.

### Install with pip

```bash
# Basic installation
pip install valradar

# With HTTP support (recommended)
pip install valradar[http]
```

### Install from source

```bash
# From the valradar repository
cd python
pip install -e .

# With HTTP extras
pip install -e ".[http]"
```

### Verify SDK Installation

```python
>>> import valradar
>>> valradar.__version__
'0.1.0'
```

## Plugin Dependencies

Individual plugins may have their own dependencies. Use the `install` command to install them:

```bash
# Install dependencies for a specific plugin
valradar install examples.emails
```

This reads the plugin's metadata and installs any required Python packages.

## Verifying Installation

Run these commands to verify everything is set up correctly:

```bash
# Check Valradar version
valradar --version

# List available plugins
valradar list

# Validate a plugin
valradar doctor examples.emails

# Run a quick test
valradar run examples.emails https://example.com
```

## Troubleshooting

### "command not found: valradar"

Make sure the Valradar binary is in your PATH:

```bash
# Check if it's accessible
which valradar

# If not, add to PATH or use full path
export PATH="$PATH:/path/to/valradar/directory"
```

### "No module named 'valradar'"

Install the Python SDK:

```bash
pip install valradar
```

### Build errors with Rust

Ensure you have a recent Rust version:

```bash
rustup update stable
rustc --version  # Should be 1.70+
```

### Plugin import errors

Make sure you're running Valradar from a directory where it can find the plugins, or configure the module search paths appropriately.

## Next Steps

With Valradar installed, head to the [Quick Start](./quick-start.md) guide to run your first plugin!
