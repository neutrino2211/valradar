"""Valradar SDK - Core classes for building scanning modules."""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Iterator, Union


@dataclass
class Option:
    """Defines a command-line option for a module.

    Attributes:
        name: Option name (used as CLI argument and kwarg key)
        type: Type hint string ("str", "int", "bool", "float")
        default: Default value if not provided
        required: Whether this option must be specified
        help: Help text for CLI
    """
    name: str
    type: str = "str"
    default: Any = None
    required: bool = False
    help: str = ""


@dataclass
class Result:
    """Yielded when a finding is discovered.

    Attributes:
        data: Dictionary of finding data (will be displayed as table columns)
        host: The target/host where this result was found
    """
    data: dict = field(default_factory=dict)
    host: str = ""

    def __post_init__(self):
        # Ensure data values are strings for table display
        self.data = {k: str(v) if not isinstance(v, str) else v
                     for k, v in self.data.items()}


@dataclass
class Task:
    """Yielded to request new work from the engine.

    The Rust engine will deduplicate tasks and manage the work queue.

    Attributes:
        target: The target string to pass to run()
        kwargs: Additional keyword arguments for run()
    """
    target: str
    kwargs: dict = field(default_factory=dict)


class Module(ABC):
    """Base class for all Valradar scanning modules.

    Subclass this to create a new module. Override `run()` to define
    your scanning logic. Use `setup()` for one-time initialization.

    Example:
        class MyScanner(Module):
            name = "My Scanner"
            description = "Scans for things"
            options = [Option("url", required=True)]

            def setup(self):
                self.session = requests.Session()

            def run(self, target, **kwargs):
                resp = self.session.get(target)
                if "secret" in resp.text:
                    yield Result(host=target, data={"found": "secret"})
                for link in extract_links(resp.text):
                    yield Task(target=link)

        MODULE_CLASS = MyScanner
    """

    # Module metadata - override in subclass
    name: str = "Unnamed Module"
    description: str = ""
    author: str = ""
    version: str = "0.1.0"
    options: list = field(default_factory=list) if False else []  # type: ignore

    def setup(self) -> None:
        """One-time initialization hook.

        Called once after instantiation, before any run() calls.
        Use for setting up sessions, loading resources, etc.
        """
        pass

    @abstractmethod
    def run(self, target: str, **kwargs) -> Iterator[Union[Result, Task]]:
        """Main scanning logic - must be implemented by subclass.

        Args:
            target: The target to scan (URL, IP, file path, etc.)
            **kwargs: Additional arguments from Task.kwargs or CLI options

        Yields:
            Result: When a finding is discovered
            Task: To request scanning of a new target

        Note:
            - Do NOT manage recursion - the Rust engine handles that
            - Do NOT track visited targets - the engine deduplicates
            - Just yield what you find and what to scan next
        """
        pass

    def get_metadata(self) -> dict:
        """Returns module metadata as a dictionary."""
        return {
            "name": self.name,
            "description": self.description,
            "author": self.author,
            "version": self.version,
            "options": [
                {
                    "name": opt.name,
                    "type": opt.type,
                    "default": opt.default,
                    "required": opt.required,
                    "help": opt.help,
                }
                for opt in (self.options or [])
            ],
        }
