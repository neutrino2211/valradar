"""Valradar Python SDK for plugin development.

This SDK provides a class-based interface for writing Valradar plugins
with better type hints, IDE support, and developer experience.

Example:
    from valradar import Plugin, Context
    
    class MyPlugin(Plugin):
        name = "my-plugin"
        description = "My awesome plugin"
        
        def init(self, args: list[str]) -> list[Context]:
            return [Context(url=url) for url in args]
        
        def collect(self, ctx: Context) -> list[Context]:
            return []
        
        def process(self, ctx: Context) -> dict[str, Any] | None:
            return {"url": ctx.get("url")}
    
    # Export for Valradar runtime
    plugin = MyPlugin()
    VALRADAR_CONFIG = to_config(plugin)
"""

from typing import Any, Callable

from .context import Context
from .plugin import Plugin

__version__ = "0.1.0"
__all__ = ["Plugin", "Context", "to_config"]


def to_config(plugin: Plugin) -> dict[str, Any]:
    """Convert a Plugin instance to the VALRADAR_CONFIG dict format.
    
    This adapter function allows class-based plugins to work with the
    existing Rust runtime which expects the old dict-based config format.
    
    Args:
        plugin: A Plugin instance to convert.
        
    Returns:
        A VALRADAR_CONFIG-compatible dictionary.
        
    Example:
        class MyPlugin(Plugin):
            ...
        
        plugin = MyPlugin()
        VALRADAR_CONFIG = to_config(plugin)
    """
    return {
        "init": plugin.init,
        "collect_data": plugin.collect,
        "process_data": plugin.process,
        "metadata": {
            "name": plugin.name,
            "description": plugin.description,
            "version": plugin.version,
            "author": plugin.author,
            "tags": plugin.tags,
        },
    }
