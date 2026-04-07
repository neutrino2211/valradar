"""Context class for Valradar plugins."""

from typing import Any, Optional

try:
    import requests
    _HAS_REQUESTS = True
except ImportError:
    _HAS_REQUESTS = False


class Context:
    """Generic data container for plugin workflows.
    
    A Context holds arbitrary data that flows through the plugin pipeline.
    Use it to pass state between init(), collect(), and process() methods.
    
    Example:
        ctx = Context(url="https://example.com")
        ctx.set("content", "<html>...</html>")
        if ctx.has("content"):
            html = ctx.get("content")
    """
    
    def __init__(self, **kwargs: Any) -> None:
        """Initialize context with optional keyword arguments.
        
        Args:
            **kwargs: Initial data to store in the context.
        """
        self.data: dict[str, Any] = kwargs
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a value from the context.
        
        Args:
            key: The key to look up.
            default: Value to return if key is not found.
            
        Returns:
            The value associated with key, or default if not found.
        """
        return self.data.get(key, default)
    
    def set(self, key: str, value: Any) -> "Context":
        """Set a value in the context.
        
        Args:
            key: The key to set.
            value: The value to store.
            
        Returns:
            Self for method chaining.
        """
        self.data[key] = value
        return self
    
    def has(self, key: str) -> bool:
        """Check if a key exists in the context.
        
        Args:
            key: The key to check.
            
        Returns:
            True if the key exists, False otherwise.
        """
        return key in self.data
    
    def fetch(self, url: str, **kwargs: Any) -> "requests.Response":
        """Fetch a URL and return the response.
        
        This is a convenience method that provides a built-in HTTP client.
        Requires the 'requests' package to be installed.
        
        Args:
            url: The URL to fetch.
            **kwargs: Additional arguments passed to requests.get().
            
        Returns:
            The requests Response object.
            
        Raises:
            ImportError: If requests is not installed.
            requests.RequestException: If the request fails.
        """
        if not _HAS_REQUESTS:
            raise ImportError(
                "The 'requests' package is required for fetch(). "
                "Install it with: pip install valradar[http]"
            )
        
        # Default headers if not provided
        if "headers" not in kwargs:
            kwargs["headers"] = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                              "AppleWebKit/537.36 (KHTML, like Gecko) "
                              "Chrome/120.0.0.0 Safari/537.36"
            }
        
        return requests.get(url, **kwargs)
    
    def __repr__(self) -> str:
        """Return a string representation for debugging."""
        items = ", ".join(f"{k}={v!r}" for k, v in self.data.items())
        return f"Context({items})"
    
    def __str__(self) -> str:
        """Return a human-readable string representation."""
        return repr(self)
