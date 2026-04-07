# API Reference

Complete reference for the Valradar Python SDK.

## Module: valradar

```python
from valradar import Plugin, Context, to_config
```

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `__version__` | `"0.1.0"` | SDK version string |

### Exports

| Name | Type | Description |
|------|------|-------------|
| `Plugin` | class | Abstract base class for plugins |
| `Context` | class | Data container for plugin workflows |
| `to_config` | function | Convert Plugin instance to VALRADAR_CONFIG format |

---

## Class: Plugin

Abstract base class that all plugins must inherit from.

```python
from valradar import Plugin
```

### Class Attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `name` | `str` | `"unnamed"` | Short identifier for the plugin |
| `description` | `str` | `""` | Brief description of what the plugin does |
| `version` | `str` | `"0.1.0"` | Semantic version string |
| `author` | `str` | `""` | Author name and contact |
| `tags` | `list[str]` | `[]` | Categorization tags |

### Abstract Methods

These methods must be implemented by every plugin:

#### init(args)

Initialize the plugin with command-line arguments.

```python
@abstractmethod
def init(self, args: list[str]) -> list[Context]:
    """Initialize the plugin.
    
    Args:
        args: Command-line arguments passed after the plugin name.
        
    Returns:
        List of Context objects to begin processing.
    """
    pass
```

**Parameters:**
- `args` (`list[str]`) — Arguments from the command line

**Returns:**
- `list[Context]` — Initial contexts to process

**Example:**

```python
def init(self, args: list[str]) -> list[Context]:
    if not args:
        print("Usage: valradar run my-plugin <url>")
        exit(1)
    return [Context(url=url) for url in args]
```

---

#### collect(ctx)

Collect data from a context and optionally spawn new contexts.

```python
@abstractmethod
def collect(self, ctx: Context) -> list[Context]:
    """Collect data from a context.
    
    Args:
        ctx: The context to collect data from.
        
    Returns:
        List of new Context objects for recursive processing.
    """
    pass
```

**Parameters:**
- `ctx` (`Context`) — The context to process

**Returns:**
- `list[Context]` — New contexts for recursive processing (can be empty)

**Example:**

```python
def collect(self, ctx: Context) -> list[Context]:
    response = ctx.fetch(ctx.get("url"))
    ctx.set("content", response.text)
    
    links = self.extract_links(response.text)
    return [Context(url=link) for link in links]
```

---

#### process(ctx)

Process collected data into final results.

```python
@abstractmethod
def process(self, ctx: Context) -> dict[str, Any] | None:
    """Process a context into results.
    
    Args:
        ctx: The context with collected data.
        
    Returns:
        Result dictionary, or None to skip this context.
    """
    pass
```

**Parameters:**
- `ctx` (`Context`) — The context to process

**Returns:**
- `dict[str, Any] | None` — Results dictionary, or `None` to skip

**Example:**

```python
def process(self, ctx: Context) -> dict[str, Any] | None:
    emails = ctx.get("emails", [])
    if emails:
        return {
            "url": ctx.get("url"),
            "count": len(emails),
            "emails": ", ".join(emails)
        }
    return None
```

---

## Class: Context

Generic data container for plugin workflows.

```python
from valradar import Context
```

### Constructor

```python
Context(**kwargs: Any)
```

Create a new context with optional initial data.

**Parameters:**
- `**kwargs` — Key-value pairs to store in the context

**Example:**

```python
ctx = Context(url="https://example.com", depth=1)
```

### Instance Attributes

| Attribute | Type | Description |
|-----------|------|-------------|
| `data` | `dict[str, Any]` | Internal data storage (prefer using methods) |

### Methods

#### get(key, default=None)

Retrieve a value from the context.

```python
def get(self, key: str, default: Any = None) -> Any:
    """Get a value from the context.
    
    Args:
        key: The key to look up.
        default: Value to return if key not found.
        
    Returns:
        The stored value, or default if not found.
    """
```

**Parameters:**
- `key` (`str`) — The key to look up
- `default` (`Any`, optional) — Fallback value. Default: `None`

**Returns:**
- `Any` — The stored value or default

**Example:**

```python
url = ctx.get("url")
count = ctx.get("count", 0)  # Returns 0 if not set
```

---

#### set(key, value)

Store a value in the context.

```python
def set(self, key: str, value: Any) -> Context:
    """Set a value in the context.
    
    Args:
        key: The key to set.
        value: The value to store.
        
    Returns:
        Self for method chaining.
    """
```

**Parameters:**
- `key` (`str`) — The key to set
- `value` (`Any`) — The value to store

**Returns:**
- `Context` — Self (for method chaining)

**Example:**

```python
ctx.set("content", html)

# Method chaining
ctx.set("status", 200).set("content", html).set("fetched", True)
```

---

#### has(key)

Check if a key exists in the context.

```python
def has(self, key: str) -> bool:
    """Check if a key exists.
    
    Args:
        key: The key to check.
        
    Returns:
        True if the key exists, False otherwise.
    """
```

**Parameters:**
- `key` (`str`) — The key to check

**Returns:**
- `bool` — Whether the key exists

**Example:**

```python
if ctx.has("error"):
    handle_error(ctx.get("error"))
```

---

#### fetch(url, **kwargs)

Convenience method to fetch a URL.

```python
def fetch(self, url: str, **kwargs: Any) -> requests.Response:
    """Fetch a URL and return the response.
    
    Args:
        url: The URL to fetch.
        **kwargs: Additional arguments for requests.get().
        
    Returns:
        The requests Response object.
        
    Raises:
        ImportError: If requests is not installed.
        requests.RequestException: If the request fails.
    """
```

**Parameters:**
- `url` (`str`) — The URL to fetch
- `**kwargs` — Additional arguments passed to `requests.get()`

**Returns:**
- `requests.Response` — The response object

**Raises:**
- `ImportError` — If `requests` package is not installed
- `requests.RequestException` — If the HTTP request fails

**Default behavior:**
- Sets a reasonable User-Agent header if none provided

**Example:**

```python
# Basic fetch
response = ctx.fetch("https://example.com")
print(response.status_code)
print(response.text)

# With options
response = ctx.fetch(
    "https://api.example.com",
    timeout=10,
    headers={"Authorization": "Bearer token"}
)
data = response.json()
```

**Requirements:**

```bash
pip install valradar[http]
# or
pip install requests
```

---

### Special Methods

#### \_\_repr\_\_() / \_\_str\_\_()

String representation for debugging.

```python
ctx = Context(url="https://example.com", count=5)
print(ctx)  # Context(url='https://example.com', count=5)
```

---

## Function: to_config

Convert a Plugin instance to VALRADAR_CONFIG format.

```python
def to_config(plugin: Plugin) -> dict[str, Any]:
    """Convert a Plugin instance to VALRADAR_CONFIG format.
    
    Args:
        plugin: A Plugin instance to convert.
        
    Returns:
        A VALRADAR_CONFIG-compatible dictionary.
    """
```

**Parameters:**
- `plugin` (`Plugin`) — The plugin instance to convert

**Returns:**
- `dict[str, Any]` — Configuration dictionary for the Rust runtime

**Example:**

```python
class MyPlugin(Plugin):
    name = "my-plugin"
    # ... implement methods ...

plugin = MyPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

**Required:** Every plugin file must end with this export pattern.

---

## VALRADAR_CONFIG Format

The `to_config()` function produces this structure:

```python
{
    "init": <callable>,           # Plugin.init method
    "collect_data": <callable>,   # Plugin.collect method
    "process_data": <callable>,   # Plugin.process method
    "metadata": {
        "name": str,              # Plugin.name
        "description": str,       # Plugin.description
        "version": str,           # Plugin.version
        "author": str,            # Plugin.author
        "tags": list[str],        # Plugin.tags
    }
}
```

This format is consumed by the Rust runtime.

---

## Type Hints Summary

```python
from typing import Any
from valradar import Plugin, Context

# Plugin methods
def init(self, args: list[str]) -> list[Context]: ...
def collect(self, ctx: Context) -> list[Context]: ...
def process(self, ctx: Context) -> dict[str, Any] | None: ...

# Context methods
def get(self, key: str, default: Any = None) -> Any: ...
def set(self, key: str, value: Any) -> Context: ...
def has(self, key: str) -> bool: ...
def fetch(self, url: str, **kwargs: Any) -> requests.Response: ...
```

---

## Complete Example

```python
"""Complete example showing all SDK features."""

from typing import Any
from valradar import Plugin, Context, to_config


class ExamplePlugin(Plugin):
    """A complete example plugin."""
    
    name = "example"
    description = "Demonstrates all SDK features"
    version = "1.0.0"
    author = "Valradar Team"
    tags = ["example", "demo"]
    
    def init(self, args: list[str]) -> list[Context]:
        if not args:
            print("Usage: valradar run example <url>")
            exit(1)
        return [Context(url=url, depth=0) for url in args]
    
    def collect(self, ctx: Context) -> list[Context]:
        url = ctx.get("url")
        depth = ctx.get("depth", 0)
        
        try:
            response = ctx.fetch(url)
            ctx.set("content", response.text)
            ctx.set("status", response.status_code)
        except Exception as e:
            ctx.set("error", str(e))
            return []
        
        # Return new contexts for recursive processing
        links = self._find_links(ctx.get("content"))
        return [
            Context(url=link, depth=depth + 1, parent=url)
            for link in links[:5]  # Limit to 5 links
        ]
    
    def process(self, ctx: Context) -> dict[str, Any] | None:
        if ctx.has("error"):
            return {
                "url": ctx.get("url"),
                "error": ctx.get("error")
            }
        
        return {
            "url": ctx.get("url")[:60],
            "status": ctx.get("status"),
            "size": len(ctx.get("content", "")),
            "depth": ctx.get("depth")
        }
    
    def _find_links(self, html: str) -> list[str]:
        """Extract links from HTML (simplified)."""
        import re
        pattern = r'href=["\']([^"\']+)["\']'
        return re.findall(pattern, html)


# Required export
plugin = ExamplePlugin()
VALRADAR_CONFIG = to_config(plugin)
```
