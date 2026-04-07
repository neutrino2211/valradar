# Writing Plugins

This section covers everything you need to know about creating Valradar plugins.

## Overview

Valradar plugins are Python modules that follow a specific interface. The SDK provides a class-based API that makes plugin development intuitive and type-safe.

## Plugin Lifecycle

Every plugin goes through three phases:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│   init   │ ──▶ │ collect  │ ──▶ │ process  │
└──────────┘     └──────────┘     └──────────┘
     │                │                │
     ▼                ▼                ▼
  Initial         Recursive         Final
  Contexts        Expansion        Results
```

1. **init()** — Called once with CLI arguments. Returns initial Context objects.
2. **collect()** — Called for each Context. Gathers data and returns new Contexts for recursive processing.
3. **process()** — Called for each Context. Transforms collected data into final results.

## Minimal Plugin Example

Here's the simplest possible plugin:

```python
from valradar import Plugin, Context, to_config

class MinimalPlugin(Plugin):
    name = "minimal"
    description = "A minimal example plugin"
    
    def init(self, args: list[str]) -> list[Context]:
        return [Context(value=arg) for arg in args]
    
    def collect(self, ctx: Context) -> list[Context]:
        return []  # No recursive collection
    
    def process(self, ctx: Context) -> dict | None:
        return {"input": ctx.get("value")}

# Required: export for Valradar runtime
plugin = MinimalPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

Run it:

```bash
valradar run my_plugin hello world
```

Output:

```
┌─────────┐
│ input   │
├─────────┤
│ hello   │
│ world   │
└─────────┘
```

## Plugin Components

### 1. Class Metadata

Define your plugin's identity:

```python
class MyPlugin(Plugin):
    name = "my-plugin"              # Short identifier
    description = "Does something"   # Brief description
    version = "1.0.0"               # Semantic version
    author = "Your Name"            # Author info
    tags = ["web", "scraping"]      # Categorization tags
```

### 2. The init() Method

Initialize the plugin and create starting contexts:

```python
def init(self, args: list[str]) -> list[Context]:
    """Called once with command-line arguments."""
    if not args:
        print("Usage: valradar run my_plugin <url>")
        exit(1)
    
    return [Context(url=url) for url in args]
```

### 3. The collect() Method

Gather data and optionally spawn new contexts:

```python
def collect(self, ctx: Context) -> list[Context]:
    """Called for each context to collect data."""
    url = ctx.get("url")
    
    # Fetch and store data
    response = ctx.fetch(url)
    ctx.set("content", response.text)
    
    # Return new contexts for recursive processing
    links = extract_links(response.text)
    return [Context(url=link) for link in links]
```

### 4. The process() Method

Transform collected data into results:

```python
def process(self, ctx: Context) -> dict | None:
    """Called for each context to produce results."""
    content = ctx.get("content", "")
    results = analyze(content)
    
    if results:
        return {
            "url": ctx.get("url"),
            "findings": results
        }
    
    return None  # Skip this context
```

## The to_config() Function

The `to_config()` function converts your plugin class into the format expected by the Rust runtime:

```python
from valradar import to_config

plugin = MyPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

This is **required** at the end of every plugin file.

## Data Flow Example

Here's how data flows through a plugin:

```python
# User runs: valradar run my_plugin https://a.com https://b.com -d 2

# Phase 1: init()
# Input: ["https://a.com", "https://b.com"]
# Output: [Context(url="https://a.com"), Context(url="https://b.com")]

# Phase 2: collect() - Depth 1
# For Context(url="https://a.com"):
#   - Fetches page, finds links to /page1 and /page2
#   - Returns [Context(url="https://a.com/page1"), Context(url="https://a.com/page2")]

# Phase 2: collect() - Depth 2
# For Context(url="https://a.com/page1"):
#   - Fetches page, finds more links
#   - Returns new contexts (processed at depth 2)

# Phase 3: process()
# All contexts are processed and results are collected
```

## Best Practices

### Return Empty Lists, Not None

In `collect()`, always return a list (empty is fine):

```python
# Good
def collect(self, ctx: Context) -> list[Context]:
    return []

# Avoid (may cause issues)
def collect(self, ctx: Context) -> list[Context]:
    return None  # Don't do this
```

### Handle Errors Gracefully

```python
def collect(self, ctx: Context) -> list[Context]:
    try:
        response = ctx.fetch(ctx.get("url"))
        ctx.set("content", response.text)
    except Exception as e:
        print(f"Error: {e}")
        ctx.set("error", str(e))
    
    return []
```

### Use Type Hints

The SDK is fully typed. Use type hints for better IDE support:

```python
from typing import Any

def process(self, ctx: Context) -> dict[str, Any] | None:
    ...
```

## Next Steps

- [Plugin Structure](./plugin-structure.md) — Deep dive into the Plugin class
- [The Context Class](./context.md) — Learn about data storage and the fetch() method
- [Examples](./examples.md) — Study real-world plugin examples
