# Plugin Structure

This page provides a detailed reference for the `Plugin` class and plugin file structure.

## The Plugin Class

The `Plugin` class is an abstract base class that defines the interface all plugins must implement.

```python
from abc import ABC, abstractmethod
from typing import Any
from valradar import Context

class Plugin(ABC):
    # Metadata attributes
    name: str = "unnamed"
    description: str = ""
    version: str = "0.1.0"
    author: str = ""
    tags: list[str] = []
    
    @abstractmethod
    def init(self, args: list[str]) -> list[Context]:
        """Initialize plugin with CLI arguments."""
        pass
    
    @abstractmethod
    def collect(self, ctx: Context) -> list[Context]:
        """Collect data from a context."""
        pass
    
    @abstractmethod
    def process(self, ctx: Context) -> dict[str, Any] | None:
        """Process a context into results."""
        pass
```

## Metadata Attributes

### name

The short identifier for your plugin. Used in CLI output and logging.

```python
name = "email-extractor"
```

**Guidelines:**
- Use lowercase with hyphens
- Keep it short and descriptive
- Should be unique within your plugin collection

### description

A brief description of what the plugin does.

```python
description = "Extract email addresses from web pages"
```

**Guidelines:**
- One sentence, no period at the end
- Focus on the action (verb + object)
- Shown in `valradar list` and `valradar run -i`

### version

Semantic version string for the plugin.

```python
version = "1.2.3"
```

**Guidelines:**
- Follow [semver](https://semver.org/) conventions
- Increment appropriately for changes

### author

Author name and optionally contact info.

```python
author = "Jane Doe <jane@example.com>"
```

### tags

List of categorization tags for discovery and filtering.

```python
tags = ["web", "scraping", "email", "osint"]
```

**Common tags:**
- `web` — Web-related operations
- `scraping` — Data extraction
- `osint` — Open-source intelligence
- `recon` — Reconnaissance
- `security` — Security-focused
- `api` — API interactions

## Method Reference

### init(args: list[str]) -> list[Context]

Called once when the plugin starts. Receives command-line arguments.

**Parameters:**
- `args` — List of strings passed after the plugin name

**Returns:**
- List of `Context` objects to begin processing

**Example:**

```python
def init(self, args: list[str]) -> list[Context]:
    # Validate arguments
    if not args:
        print("Error: At least one URL required")
        exit(1)
    
    # Create initial contexts
    contexts = []
    for url in args:
        if url.startswith("http"):
            contexts.append(Context(url=url))
        else:
            print(f"Skipping invalid URL: {url}")
    
    return contexts
```

### collect(ctx: Context) -> list[Context]

Called for each context during the collection phase. This is where you fetch data and optionally spawn new contexts for recursive processing.

**Parameters:**
- `ctx` — The context to collect data from

**Returns:**
- List of new `Context` objects (can be empty)

**Example:**

```python
def collect(self, ctx: Context) -> list[Context]:
    url = ctx.get("url")
    
    try:
        # Fetch the page
        response = ctx.fetch(url)
        content = response.text
        
        # Store collected data
        ctx.set("content", content)
        ctx.set("status_code", response.status_code)
        
        # Extract links for recursive crawling
        new_contexts = []
        for link in self.find_links(content):
            new_contexts.append(Context(url=link, parent=url))
        
        return new_contexts
        
    except Exception as e:
        ctx.set("error", str(e))
        return []
```

**Important notes:**
- Always return a list (empty if no new contexts)
- Store collected data in the context using `ctx.set()`
- The runtime controls recursion depth; just return contexts

### process(ctx: Context) -> dict[str, Any] | None

Called for each context during the processing phase. Transform collected data into final results.

**Parameters:**
- `ctx` — The context to process

**Returns:**
- Dictionary with results, or `None` to skip this context

**Example:**

```python
def process(self, ctx: Context) -> dict[str, Any] | None:
    # Check for errors
    if ctx.has("error"):
        return None
    
    content = ctx.get("content", "")
    emails = self.extract_emails(content)
    
    # Only return results if we found something
    if emails:
        return {
            "url": ctx.get("url"),
            "email_count": len(emails),
            "emails": ", ".join(emails)
        }
    
    return None
```

**Result dictionary:**
- Keys become column headers in output
- Values should be simple types (str, int, float, bool)
- Complex objects are converted to strings

## File Structure

A complete plugin file should have this structure:

```python
"""Plugin docstring explaining what it does."""

# Standard library imports
import re
from typing import Any

# Third-party imports
import some_library

# Valradar imports
from valradar import Plugin, Context, to_config


class MyPlugin(Plugin):
    """Main plugin class."""
    
    # Metadata
    name = "my-plugin"
    description = "Does something useful"
    version = "1.0.0"
    author = "Your Name"
    tags = ["category"]
    
    def init(self, args: list[str]) -> list[Context]:
        """Initialize with arguments."""
        return [Context(data=arg) for arg in args]
    
    def collect(self, ctx: Context) -> list[Context]:
        """Collect data."""
        # Your collection logic
        return []
    
    def process(self, ctx: Context) -> dict[str, Any] | None:
        """Process data into results."""
        # Your processing logic
        return {"result": ctx.get("data")}
    
    # Helper methods (optional)
    def _helper_method(self, data: str) -> str:
        """Private helper method."""
        return data.strip()


# Required: Export for Valradar runtime
plugin = MyPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

## Legacy Format

For reference, the legacy dict-based format is still supported but not recommended:

```python
# Legacy format (not recommended)
def _VALRADAR_INIT(args):
    return [{"url": url} for url in args]

def _VALRADAR_COLLECT_DATA(context):
    return []

def _VALRADAR_PROCESS_DATA(context):
    return {"url": context["url"]}

VALRADAR_CONFIG = {
    "init": _VALRADAR_INIT,
    "collect_data": _VALRADAR_COLLECT_DATA,
    "process_data": _VALRADAR_PROCESS_DATA,
    "metadata": {
        "name": "legacy-plugin",
        "description": "Old-style plugin",
        "version": "0.1.0",
    }
}
```

The class-based approach provides better:
- Type safety and IDE support
- Code organization
- Testability
- Documentation

## Next Steps

- [The Context Class](./context.md) — Learn about the Context API
- [Examples](./examples.md) — See complete plugin examples
