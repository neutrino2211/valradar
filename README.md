# Valradar

Valradar is a high-performance, low-latency, and scalable data processing framework designed for OSINT and RECON operations. It provides a flexible plugin architecture that allows you to create custom data collection and processing pipelines.

## Features

- **Plugin-based Architecture**: Create custom data processing plugins in Python
- **High Performance**: Multiprocessing support for parallel data processing
- **State Management**: Built-in support for maintaining state across processing steps
- **Recursive Processing**: Support for depth-based recursive data collection
- **Extensible**: Easy to extend with new plugins and capabilities
- **Command-line Interface**: Simple CLI for running plugins with various options

## Installation

```bash
cargo build --release
```

## Usage

Run a plugin using the following command:

```bash
cargo run -- -c 4 examples.emails https://example.com
```

### Command Line Options

- `-c, --concurrency`: Number of concurrent worker threads (default: 1)
- `-d, --depth`: How many recursive calls to make (default: 1)
- `-!, --debug`: Enable debug mode (default: false)
- `plugin`: Plugin module name (e.g., examples.emails)
- `args`: Arguments for the plugin

## Creating Plugins

Plugins in Valradar are Python modules that implement a specific interface. Here's how to create one:

### Plugin Structure

A Valradar plugin consists of three main components:

1. A `DataContext` class to manage state
2. Required plugin functions
3. Plugin configuration

### Example Plugin

Here's a simplified example of an email extraction plugin:

```python
class DataContext:
    def __init__(self, url):
        self.url = url
        self.data = {}
        self.processed = False
        self.emails = []

    def collect(self):
        # Collect data and return new contexts for recursive processing
        # This method is called for each data item
        return [DataContext(new_url) for new_url in self.extract_links()]

    def process(self):
        # Process the collected data
        # Return None if no results, or a dict with results
        if len(self.emails) > 0:
            return {"url": self.url, "emails": self.emails}
        return None

# Required plugin functions
def _VALRADAR_INIT(args):
    # Initialize plugin with arguments
    return [DataContext(url) for url in args]

def _VALRADAR_COLLECT_DATA(context):
    # Collect data from a context
    return context.collect()

def _VALRADAR_PROCESS_DATA(context):
    # Process data from a context
    return context.process()

# Plugin configuration
VALRADAR_CONFIG = {
    "init": _VALRADAR_INIT,
    "collect_data": _VALRADAR_COLLECT_DATA,
    "process_data": _VALRADAR_PROCESS_DATA,
    "metadata": {
        "name": "Plugin Name",
        "description": "Plugin description",
        "version": "0.1.0",
        "tags": ["tag1", "tag2"],
        "author": "Your Name",
        "license": "MIT",
        "dependencies": ["dependency1", "dependency2"],
        "requirements": ["requirement1", "requirement2"]
    }
}
```

### Required Functions

1. `_VALRADAR_INIT(args)`: 
   - Initializes the plugin with command-line arguments
   - Returns a list of initial `DataContext` objects

2. `_VALRADAR_COLLECT_DATA(context)`:
   - Called for each data item to collect new data
   - Returns a list of new `DataContext` objects for recursive processing

3. `_VALRADAR_PROCESS_DATA(context)`:
   - Processes the collected data
   - Returns None if no results, or a dictionary with results

### DataContext Class

The `DataContext` class is used to maintain state during processing:

- `__init__`: Initialize the context with input data
- `collect`: Collect new data and return new contexts
- `process`: Process the collected data and return results

### Plugin Configuration

The `VALRADAR_CONFIG` dictionary defines the plugin's interface and metadata:

- `init`: Initialization function
- `collect_data`: Data collection function
- `process_data`: Data processing function
- `metadata`: Plugin metadata including name, description, dependencies, etc.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 