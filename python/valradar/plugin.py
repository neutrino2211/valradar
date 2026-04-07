"""Base Plugin class for Valradar plugins."""

from abc import ABC, abstractmethod
from typing import Any

from .context import Context


class Plugin(ABC):
    """Abstract base class for Valradar plugins.
    
    Subclass this to create a new plugin. You must implement:
    - init(): Initialize with CLI arguments, return initial contexts
    - collect(): Collect data from a context, return new contexts
    - process(): Process a context into a final result
    
    Example:
        from valradar import Plugin, Context
        
        class MyPlugin(Plugin):
            name = "my-plugin"
            description = "Does something useful"
            
            def init(self, args: list[str]) -> list[Context]:
                return [Context(url=url) for url in args]
            
            def collect(self, ctx: Context) -> list[Context]:
                # Fetch data, return new contexts to explore
                return []
            
            def process(self, ctx: Context) -> dict[str, Any] | None:
                # Return result dict or None to skip
                return {"url": ctx.get("url")}
    """
    
    # Plugin metadata - override in subclasses
    name: str = "unnamed"
    description: str = ""
    version: str = "0.1.0"
    author: str = ""
    tags: list[str] = []
    
    @abstractmethod
    def init(self, args: list[str]) -> list[Context]:
        """Initialize the plugin with CLI arguments.
        
        This is called once at startup with the command-line arguments
        passed to the plugin. Return a list of initial Context objects
        to begin processing.
        
        Args:
            args: Command-line arguments passed to the plugin.
            
        Returns:
            List of Context objects to process.
        """
        pass
    
    @abstractmethod
    def collect(self, ctx: Context) -> list[Context]:
        """Collect data from a context.
        
        This is called for each context. Perform data collection here
        (e.g., fetch a URL, parse content). Return new Context objects
        for recursive processing, or an empty list to stop recursion.
        
        Args:
            ctx: The context to collect data from.
            
        Returns:
            List of new Context objects for recursive processing.
        """
        pass
    
    @abstractmethod
    def process(self, ctx: Context) -> dict[str, Any] | None:
        """Process a context into a final result.
        
        This is called for each context after collection. Return a
        dictionary with the results, or None to skip this context.
        
        Args:
            ctx: The context to process.
            
        Returns:
            Result dictionary, or None to skip.
        """
        pass
