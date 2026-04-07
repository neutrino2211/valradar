# Valradar Python SDK

A Python SDK for writing [Valradar](https://github.com/neutrino2211/valradar) OSINT/RECON plugins with better developer experience, type hints, and IDE support.

## Installation

```bash
pip install valradar

# With HTTP client support (recommended)
pip install valradar[http]
```

## Quick Start

```python
from valradar import Plugin, Context, to_config
from typing import Any

class MyPlugin(Plugin):
    name = "my-plugin"
    description = "Extracts data from websites"
    version = "0.1.0"
    author = "Your Name"
    tags = ["web", "scraping"]
    
    def init(self, args: list[str]) -> list[Context]:
        """Initialize with URLs from command line."""
        if not args:
            print("Usage: valradar my-plugin <url> [url...]")
            exit(1)
        return [Context(url=url) for url in args]
    
    def collect(self, ctx: Context) -> list[Context]:
        """Fetch URL and find more links to explore."""
        url = ctx.get("url")
        response = ctx.fetch(url)
        ctx.set("content", response.text)
        # Return new contexts for recursive processing
        return []
    
    def process(self, ctx: Context) -> dict[str, Any] | None:
        """Extract and return results."""
        if ctx.has("content"):
            return {"url": ctx.get("url"), "length": len(ctx.get("content"))}
        return None

# Export for Valradar runtime
plugin = MyPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

## API Reference

### Plugin (Base Class)

The `Plugin` class is the base for all Valradar plugins. Override these methods:

| Method | Description |
|--------|-------------|
| `init(args)` | Called once with CLI args. Return initial `Context` list. |
| `collect(ctx)` | Called per context. Collect data, return new contexts. |
| `process(ctx)` | Called per context. Return result dict or `None`. |

Plugin metadata attributes:

```python
class MyPlugin(Plugin):
    name = "plugin-name"        # Plugin identifier
    description = "..."         # What it does
    version = "0.1.0"          # Semantic version
    author = "Your Name"       # Author name
    tags = ["tag1", "tag2"]    # Searchable tags
```

### Context

A generic data container that flows through the pipeline.

```python
ctx = Context(url="https://example.com", depth=0)

# Get/set values
ctx.get("url")                    # "https://example.com"
ctx.get("missing", "default")     # "default"
ctx.set("content", "<html>...")   # Returns self for chaining
ctx.has("url")                    # True

# Built-in HTTP client (requires valradar[http])
response = ctx.fetch("https://example.com")
response.text  # HTML content
```

### to_config()

Converts a `Plugin` instance to the legacy `VALRADAR_CONFIG` dict format for compatibility with the Rust runtime.

```python
plugin = MyPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

## Running Plugins

```bash
# Run with valradar CLI
valradar my-plugin https://example.com

# Or with cargo during development
cargo run -- my-plugin https://example.com
```

## Development

```bash
# Clone and install in development mode
git clone https://github.com/neutrino2211/valradar
cd valradar/python
pip install -e ".[dev,http]"

# Run tests
pytest

# Type checking
mypy valradar

# Linting
ruff check valradar
```

## License

MIT License - see [LICENSE](../LICENSE) for details.
