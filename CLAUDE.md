# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Valradar is a Rust-based security scanning framework that executes Python plugins for OSINT, RECON, and vulnerability scanning operations. The core is written in Rust with PyO3 for Python interoperability.

## Build and Run Commands

```bash
# Build the project (requires PYO3_USE_ABI3_FORWARD_COMPATIBILITY for Python 3.13+)
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release

# Run a plugin (format: cargo run -- [options] <plugin_module_path> <targets...>)
cargo run -- modules.web.emails https://example.com
cargo run -- -c 4 -m 100 modules.cves.CVE-2025-55182 https://target.com

# Show plugin info
cargo run -- -i modules.web.emails
```

### CLI Options
- `-c, --concurrency`: Worker threads (default: 4)
- `-m, --max-tasks`: Maximum tasks to process, 0 = unlimited (default: 0)
- `-!, --debug`: Enable debug output
- `-i, --info`: Show plugin metadata
- `-l, --license`: Show license

## Architecture

### Core Components (Rust)

- **`src/main.rs`**: CLI entry point using clap. Handles argument parsing, plugin instantiation, and orchestration.
- **`src/plugin.rs`**: Python plugin interface via PyO3. Loads `MODULE_CLASS`, calls `setup()`, iterates `run()` generator.
- **`src/orchestrator.rs`**: Multithreaded worker pool with HashSet deduplication. Distributes targets and processes `Result`/`Task` yields.
- **`src/utils/context.rs`**: Core types: `YieldValue` (Result|Task enum), `TaskRequest`, `PluginInstance`, `ProcessingResult`.
- **`src/utils/module.rs`**: Module resolution - searches current directory then `~/.valradar/modules/`.

### Plugin SDK (Python)

Plugins are Python modules using the `valradar.sdk`:

```python
from valradar.sdk import Module, Option, Result, Task

class MyScanner(Module):
    name = "My Scanner"
    description = "Scans for things"
    options = [Option("url", required=True)]

    def setup(self):
        self.session = requests.Session()

    def run(self, target: str, **kwargs):
        # Yield findings
        yield Result(host=target, data={"key": "value"})
        # Yield new tasks (Rust handles dedup)
        yield Task(target="https://other-url.com")

MODULE_CLASS = MyScanner
```

### Data Flow
1. `MODULE_CLASS` instantiated, `setup()` called once
2. Initial targets seeded to work queue
3. Workers call `run(target)` and iterate generator
4. `Result` yields collected for output table
5. `Task` yields deduplicated and queued for processing
6. Continues until queue empty or `--max-tasks` reached

### Plugin Locations
- `modules/`: Production plugins organized by category (cves/, forensics/, web/)

Plugins are referenced by dot-notation path: `modules.web.emails` → `modules/web/emails.py`

## Key Dependencies

- **pyo3**: Rust-Python bindings with auto-initialize and abi3-py38
- **crossbeam**: Multi-producer multi-consumer channels for worker coordination
- **clap**: CLI argument parsing
- **indicatif**: Progress bars
