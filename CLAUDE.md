# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Valradar is a Rust-based security scanning framework that executes Python plugins for OSINT, RECON, and vulnerability scanning operations. The core is written in Rust with PyO3 for Python interoperability.

## Build and Run Commands

```bash
# Build the project
cargo build --release

# Run a plugin (format: cargo run -- [options] <plugin_module_path> [plugin_args])
cargo run -- -c 4 examples.emails https://example.com
cargo run -- -c 4 -d 2 modules.cves.CVE-2025-55182 https://target.com

# Show plugin info
cargo run -- -i examples.emails
```

### CLI Options
- `-c, --concurrency`: Worker threads (default: 4)
- `-d, --depth`: Recursive collection depth (default: 1)
- `-!, --debug`: Enable debug output
- `-i, --info`: Show plugin metadata
- `-l, --license`: Show license

## Architecture

### Core Components (Rust)

- **`src/main.rs`**: CLI entry point using clap. Handles argument parsing, plugin loading, and orchestrates the collect→process pipeline.
- **`src/plugin.rs`**: Python plugin interface via PyO3. Creates Python interpreter, loads plugin code, and exposes `init()`, `collect_data()`, `process_data()` methods.
- **`src/orchestrator.rs`**: Multithreaded worker pool using crossbeam channels. Distributes `ExecutionContext` objects across workers for parallel collection.
- **`src/utils/module.rs`**: Module resolution - searches current directory then `~/.valradar/modules/` for plugin files.

### Plugin System (Python)

Plugins are Python modules that must export a `VALRADAR_CONFIG` dict with:
- `init(args)`: Returns list of `DataContext` objects from CLI args
- `collect_data(context)`: Returns list of new contexts for recursive processing
- `process_data(context)`: Returns dict with results or None

### Data Flow
1. `init()` creates initial `DataContext` objects from CLI args
2. `Orchestrator` distributes contexts to worker threads
3. Each worker calls `collect_data()` which returns new contexts for next depth level
4. After all depths complete, `process_data()` is called sequentially on all collected contexts
5. Results displayed as a table

### Plugin Locations
- `examples/`: Example plugins (emails.py, yara-scan.py)
- `modules/`: Production plugins organized by category (cves/, forensics/, web/)

Plugins are referenced by dot-notation path: `modules.cves.CVE-2025-55182` → `modules/cves/CVE-2025-55182.py`

## Key Dependencies

- **pyo3**: Rust-Python bindings with auto-initialize
- **crossbeam**: Multi-producer multi-consumer channels for worker coordination
- **clap**: CLI argument parsing
- **indicatif**: Progress bars
