# The Context Class

The `Context` class is a flexible data container that flows through the plugin pipeline. It holds all state for a single unit of work.

## Overview

```python
from valradar import Context

# Create a context with initial data
ctx = Context(url="https://example.com", depth=1)

# Store additional data
ctx.set("content", "<html>...</html>")

# Retrieve data
url = ctx.get("url")
content = ctx.get("content")

# Check for keys
if ctx.has("content"):
    process_content(ctx.get("content"))
```

## Constructor

```python
Context(**kwargs: Any)
```

Create a new context with optional initial key-value pairs.

**Example:**

```python
# Empty context
ctx = Context()

# With initial data
ctx = Context(
    url="https://example.com",
    method="GET",
    headers={"User-Agent": "Bot/1.0"}
)
```

## Methods

### get(key, default=None)

Retrieve a value from the context.

```python
get(key: str, default: Any = None) -> Any
```

**Parameters:**
- `key` — The key to look up
- `default` — Value to return if key doesn't exist (default: `None`)

**Returns:**
- The stored value, or `default` if not found

**Example:**

```python
ctx = Context(url="https://example.com")

url = ctx.get("url")           # "https://example.com"
missing = ctx.get("content")   # None
safe = ctx.get("count", 0)     # 0
```

### set(key, value)

Store a value in the context. Returns self for method chaining.

```python
set(key: str, value: Any) -> Context
```

**Parameters:**
- `key` — The key to set
- `value` — The value to store

**Returns:**
- The context itself (for chaining)

**Example:**

```python
ctx = Context()

# Simple set
ctx.set("url", "https://example.com")

# Method chaining
ctx.set("status", 200).set("content", "<html>").set("processed", True)
```

### has(key)

Check if a key exists in the context.

```python
has(key: str) -> bool
```

**Parameters:**
- `key` — The key to check

**Returns:**
- `True` if the key exists, `False` otherwise

**Example:**

```python
ctx = Context(url="https://example.com")

ctx.has("url")      # True
ctx.has("content")  # False
```

### fetch(url, **kwargs)

Convenience method to fetch a URL using the `requests` library.

```python
fetch(url: str, **kwargs: Any) -> requests.Response
```

**Parameters:**
- `url` — The URL to fetch
- `**kwargs` — Additional arguments passed to `requests.get()`

**Returns:**
- A `requests.Response` object

**Raises:**
- `ImportError` — If `requests` is not installed
- `requests.RequestException` — If the request fails

**Example:**

```python
ctx = Context()

# Basic fetch
response = ctx.fetch("https://example.com")
print(response.status_code)  # 200
print(response.text)         # HTML content

# With custom options
response = ctx.fetch(
    "https://api.example.com/data",
    timeout=10,
    headers={"Authorization": "Bearer token123"}
)
```

**Default behavior:**
- Sends a reasonable User-Agent header if none provided
- Uses `requests.get()` under the hood

**Installing HTTP support:**

```bash
pip install valradar[http]
# or
pip install requests
```

## Internal Data Storage

The context stores all data in an internal `data` dictionary:

```python
ctx = Context(url="https://example.com")
ctx.set("content", "<html>")

# Access internal dict (not recommended)
print(ctx.data)  # {"url": "https://example.com", "content": "<html>"}
```

> **Note:** While you can access `ctx.data` directly, using `get()`, `set()`, and `has()` is preferred for consistency.

## String Representation

Contexts have helpful string representations for debugging:

```python
ctx = Context(url="https://example.com", count=5)

print(ctx)       # Context(url='https://example.com', count=5)
print(repr(ctx)) # Context(url='https://example.com', count=5)
```

## Usage Patterns

### Passing State Through Pipeline

```python
def init(self, args):
    return [Context(url=url, source="cli") for url in args]

def collect(self, ctx):
    response = ctx.fetch(ctx.get("url"))
    ctx.set("content", response.text)
    ctx.set("status", response.status_code)
    return []

def process(self, ctx):
    return {
        "url": ctx.get("url"),
        "status": ctx.get("status"),
        "length": len(ctx.get("content", ""))
    }
```

### Recursive Context Creation

```python
def collect(self, ctx):
    url = ctx.get("url")
    depth = ctx.get("depth", 0)
    
    response = ctx.fetch(url)
    ctx.set("content", response.text)
    
    # Create child contexts with incremented depth
    links = self.extract_links(response.text)
    return [
        Context(url=link, depth=depth + 1, parent=url)
        for link in links
    ]
```

### Error Handling

```python
def collect(self, ctx):
    try:
        response = ctx.fetch(ctx.get("url"))
        ctx.set("content", response.text)
        ctx.set("success", True)
    except Exception as e:
        ctx.set("error", str(e))
        ctx.set("success", False)
    
    return []

def process(self, ctx):
    if not ctx.get("success"):
        return {
            "url": ctx.get("url"),
            "error": ctx.get("error")
        }
    
    # Process successful fetch
    ...
```

### Storing Complex Data

```python
def collect(self, ctx):
    response = ctx.fetch(ctx.get("url"))
    
    # Store structured data
    ctx.set("response", {
        "status": response.status_code,
        "headers": dict(response.headers),
        "cookies": dict(response.cookies),
        "body": response.text[:10000]  # Limit size
    })
    
    return []
```

## Best Practices

### Use Meaningful Keys

```python
# Good
ctx.set("email_addresses", emails)
ctx.set("page_content", html)

# Avoid
ctx.set("data", emails)
ctx.set("x", html)
```

### Provide Defaults

```python
# Safe - always works
count = ctx.get("count", 0)
items = ctx.get("items", [])

# Risky - might be None
count = ctx.get("count")  # Could be None
```

### Keep Contexts Lightweight

```python
# Good - store only what you need
ctx.set("emails", extracted_emails)

# Avoid - storing entire response objects
ctx.set("response", response)  # May not serialize well
```

### Use has() for Conditional Logic

```python
# Good
if ctx.has("error"):
    handle_error(ctx)

# Works but less clear
if ctx.get("error") is not None:
    handle_error(ctx)
```

## Next Steps

- [Examples](./examples.md) — See Context in action in real plugins
- [API Reference](./api-reference.md) — Complete API documentation
