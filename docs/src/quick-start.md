# Quick Start

Run your first Valradar plugin in 5 minutes!

## Prerequisites

- Valradar binary installed ([Installation Guide](./installation.md))
- Python 3.10+ with `valradar` SDK
- `requests` and `beautifulsoup4` packages (for the email example)

## Step 1: Install Dependencies

```bash
# Install the SDK with HTTP support
pip install valradar[http] beautifulsoup4
```

## Step 2: Run the Email Extractor

The `examples.emails` plugin crawls a website and extracts email addresses:

```bash
valradar run examples.emails https://example.com
```

You should see output like:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Valradar v0.1.0                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│ Plugin: emails                                                          │
│ Description: Extract emails from a website                              │
│ Version: 0.1.0                                                          │
│ Author: Mainasara Tsowa                                                 │
└─────────────────────────────────────────────────────────────────────────┘

Processing with 4 workers, depth 1...

┌──────────────────────────────────────┬─────────────────────────────────┐
│ url                                  │ emails                          │
├──────────────────────────────────────┼─────────────────────────────────┤
│ https://example.com/contact/         │ contact@example.com             │
└──────────────────────────────────────┴─────────────────────────────────┘
```

## Step 3: Adjust Concurrency and Depth

Control how Valradar processes data:

```bash
# Use 8 concurrent workers
valradar run -c 8 examples.emails https://example.com

# Crawl 3 levels deep
valradar run -d 3 examples.emails https://example.com

# Combine both
valradar run -c 8 -d 3 examples.emails https://example.com
```

### Options Explained

| Option | Description | Default |
|--------|-------------|---------|
| `-c, --concurrency` | Number of parallel workers | 4 |
| `-d, --depth` | Recursive crawl depth | 1 |
| `-!, --debug` | Enable debug output | false |
| `-i, --info` | Show plugin info only | false |

## Step 4: View Plugin Information

Check what a plugin does before running it:

```bash
valradar run -i examples.emails
```

Output:

```
Plugin: emails
Description: Extract emails from a website
Version: 0.1.0
Author: Mainasara Tsowa
Tags: email, scraping, web
```

## Step 5: List Available Plugins

See all plugins Valradar can discover:

```bash
valradar list
```

## Step 6: Create Your Own Plugin

Generate a new plugin from template:

```bash
valradar new my-plugin
```

This creates `my-plugin.py` with the basic structure ready to customize.

## Example: Multiple URLs

Plugins can accept multiple arguments:

```bash
valradar run examples.emails https://site1.com https://site2.com https://site3.com
```

Each URL becomes an initial context, and all are processed in parallel.

## What's Next?

Now that you've run your first plugin:

- [Writing Plugins](./writing-plugins.md) — Learn how to create custom plugins
- [CLI Reference](./cli-reference.md) — Explore all available commands
- [Examples](./examples.md) — Study more plugin examples

## Troubleshooting

### "Plugin not found"

Make sure you're running from the Valradar directory, or that the plugin is in a discoverable location:

```bash
cd /path/to/valradar
valradar run examples.emails https://example.com
```

### No output / empty results

Some websites may not have emails or may block crawlers. Try a different target or check for errors with debug mode:

```bash
valradar run -! examples.emails https://example.com
```

### Import errors

Install missing dependencies:

```bash
pip install requests beautifulsoup4
# Or use the install command
valradar install examples.emails
```
